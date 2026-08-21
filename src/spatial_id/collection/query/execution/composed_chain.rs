use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::traits::UnaryOperator;
use crate::spatial_id::collection::query::working::WorkingTree;
use crate::{Error, FlexId};
use alloc::vec::Vec;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

pub fn run_composed_chain<V: SafeValue + 'static>(
    ops: &[&dyn UnaryOperator<V>],
    working: WorkingTree<V>,
    token: &CancellationToken,
) -> Result<WorkingTree<V>, Error> {
    if ops.is_empty() {
        return Ok(working);
    }
    let current_items: Vec<(FlexId, V)> = working.into_iter().collect();
    let final_items = run_composed_chain_flat(ops, current_items, token)?;
    Ok(final_items.into_iter().collect())
}

/// 演算子チェーンを中間木および中間Vecなしでダイレクト1パス（パイプライン）で実行し、フラットな Vec を返す。
pub fn run_composed_chain_flat<V: SafeValue + 'static>(
    ops: &[&dyn UnaryOperator<V>],
    input_items: Vec<(FlexId, V)>,
    token: &CancellationToken,
) -> Result<Vec<(FlexId, V)>, Error> {
    if ops.is_empty() {
        return Ok(input_items);
    }

    let mut current_items = input_items;

    let mut start_idx = 0;
    while start_idx < ops.len() {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }

        // Forward map に非対応な演算子の場合は WorkingTree へフォールバック
        if !ops[start_idx].can_forward_map() {
            let mut tree: WorkingTree<V> = current_items.into_iter().collect();
            ops[start_idx].run(&mut tree)?;
            current_items = tree.into_iter().collect();
            start_idx += 1;
            continue;
        }

        // can_forward_map == true が連続するサブグループ（区間）を抽出
        let mut end_idx = start_idx;
        while end_idx < ops.len() && ops[end_idx].can_forward_map() {
            end_idx += 1;
        }

        let sub_ops = &ops[start_idx..end_idx];

        // 1パス・パイプラインで一気に sub_ops 全体を適用
        current_items = run_pipeline_segment(sub_ops, current_items)?;

        start_idx = end_idx;
    }

    Ok(current_items)
}

/// 連続する `sub_ops`（全て forward_map 対応）を中間 Vec なしの 1 パス（パイプライン）で評価する。
fn run_pipeline_segment<V: SafeValue + 'static>(
    sub_ops: &[&dyn UnaryOperator<V>],
    input: Vec<(FlexId, V)>,
) -> Result<Vec<(FlexId, V)>, Error> {
    if sub_ops.is_empty() {
        return Ok(input);
    }

    let need_merge = sub_ops.iter().any(|op| op.collision_merge().is_some());

    #[cfg(feature = "rayon")]
    let mut final_out: Vec<(FlexId, V)> = {
        let chunk_size = (input.len() / (rayon::current_num_threads() * 4)).max(256);
        let res: Result<Vec<Vec<(FlexId, V)>>, Error> = input
            .par_chunks(chunk_size)
            .map(|chunk| {
                let mut out = Vec::with_capacity(chunk.len());
                let mut tmp_bufs: Vec<Vec<(FlexId, V)>> =
                    (0..sub_ops.len()).map(|_| Vec::with_capacity(4)).collect();

                for (id, val) in chunk {
                    apply_pipeline_item(sub_ops, *id, val.clone(), &mut out, &mut tmp_bufs)?;
                }
                Ok(out)
            })
            .collect();

        let chunk_results = res?;
        let total_len: usize = chunk_results.iter().map(|c| c.len()).sum();
        let mut combined = Vec::with_capacity(total_len);
        for mut c in chunk_results {
            combined.append(&mut c);
        }
        combined
    };

    #[cfg(not(feature = "rayon"))]
    let mut final_out: Vec<(FlexId, V)> = {
        let total_expansion: f64 = sub_ops.iter().map(|op| op.expansion_ratio()).product();
        let estimated_cap = libm::ceil(input.len() as f64 * total_expansion) as usize;
        let mut out = Vec::with_capacity(estimated_cap);
        let mut tmp_bufs: Vec<Vec<(FlexId, V)>> =
            (0..sub_ops.len()).map(|_| Vec::with_capacity(4)).collect();

        for (id, val) in input {
            apply_pipeline_item(sub_ops, id, val, &mut out, &mut tmp_bufs)?;
        }
        out
    };

    // 途中で衝突を生む演算子があった場合はソート＋マージ
    if need_merge {
        #[cfg(feature = "rayon")]
        final_out.par_sort_unstable_by_key(|(id, _)| *id);

        #[cfg(not(feature = "rayon"))]
        final_out.sort_unstable_by_key(|(id, _)| *id);

        // 最後に適用された collision_merge ポリシーで統合
        for op in sub_ops {
            if let Some(merge_fn) = op.collision_merge() {
                merge_fn(&mut final_out);
            }
        }
    }

    Ok(final_out)
}

/// 1つの (FlexId, V) を sub_ops の pipeline に通し、中間 Vec を作らず最終 out へ直接書き込む
fn apply_pipeline_item<V: SafeValue + 'static>(
    sub_ops: &[&dyn UnaryOperator<V>],
    id: FlexId,
    val: V,
    out: &mut Vec<(FlexId, V)>,
    tmp_bufs: &mut [Vec<(FlexId, V)>],
) -> Result<(), Error> {
    if sub_ops.is_empty() {
        out.push((id, val));
        return Ok(());
    }

    let op = sub_ops[0];
    if sub_ops.len() == 1 {
        // 最終演算子: 最終 out バッファへ直接出力
        op.forward_map(id, val, out)?;
    } else {
        // 中間演算子: tmp_bufs[0] と残り tmp_bufs[1..] に安全に分割
        let (first_buf, rest_bufs) = tmp_bufs.split_at_mut(1);
        let buf = &mut first_buf[0];
        buf.clear();
        op.forward_map(id, val, buf)?;

        for (next_id, next_val) in buf.iter().cloned() {
            apply_pipeline_item(&sub_ops[1..], next_id, next_val, out, rest_bufs)?;
        }
    }
    Ok(())
}

/// ソート済み `Vec` の隣接する同一 [`FlexId`] を [`crate::spatial_id::collection::query::merge_policy::MergePolicy::resolve_many`] でマージして詰める。
#[allow(clippy::collapsible_if)]
pub fn merge_sorted_vec<
    V: SafeValue,
    P: crate::spatial_id::collection::query::merge_policy::MergePolicy<V>,
>(
    data: &mut Vec<(FlexId, V)>,
) {
    if data.len() <= 1 {
        return;
    }

    let mut iter = data.drain(..);
    let mut result = Vec::with_capacity(iter.len());
    let mut group = Vec::new();
    let mut current_id = None;

    for (id, val) in iter.by_ref() {
        if Some(id) != current_id {
            if let Some(cid) = current_id {
                if let Some(merged) = P::resolve_many(group.drain(..)) {
                    result.push((cid, merged));
                }
            }
            current_id = Some(id);
        }
        group.push(val);
    }
    if let Some(cid) = current_id {
        if let Some(merged) = P::resolve_many(group.drain(..)) {
            result.push((cid, merged));
        }
    }
    drop(iter);

    *data = result;
}
