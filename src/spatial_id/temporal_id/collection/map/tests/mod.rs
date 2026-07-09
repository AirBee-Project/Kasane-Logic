use super::TemporalMap;
use crate::{ConflictPolicy, TemporalId};
use alloc::collections::BTreeMap;

/// 秒 → 値 の写像へ展開（有界ドメイン）。
fn secmap(m: &TemporalMap<i32>) -> BTreeMap<u64, i32> {
    let mut out = BTreeMap::new();
    for (s, e, v) in m.ranges_ref() {
        for sec in s..e {
            out.insert(sec, *v);
        }
    }
    out
}

fn seg(i: u64, t: u64, v: i32) -> TemporalMap<i32> {
    let mut tm = TemporalMap::new();
    tm.insert(&TemporalId::new(i, t).unwrap(), v);
    tm
}

/// 正規化不変条件（昇順・互いに素・隣接同値なし）。
fn assert_normalized(m: &TemporalMap<i32>) {
    let segments = m.ranges_ref();
    for w in segments.windows(2) {
        // 昇順・非重なり
        assert!(w[0].1 <= w[1].0, "overlap/order: {:?}", segments);
        // 隣接同値は無い
        assert!(
            !(w[0].1 == w[1].0 && w[0].2 == w[1].2),
            "adjacent equal not merged: {:?}",
            segments
        );
    }
    for &(s, e, _) in &segments {
        assert!(s < e);
    }
}

/// 秒写像正解で union/intersection/difference を照合。
#[test]
fn map_algebra_oracle() {
    // A: [0,120)=1, [180,240)=2   B: [60,200)=9
    let mut a = seg(60, 0, 1); // [0,60)=1 …作り込みは union で
    a = a.union(&seg(60, 1, 1), &ConflictPolicy::Overwrite); // [0,120)=1
    a = a.union(&seg(60, 3, 2), &ConflictPolicy::Overwrite); // +[180,240)=2
    let b = {
        // B = [60,200)=9 を秒単位で作る
        let mut bb = TemporalMap::new();
        for t in 60..200u64 {
            bb = bb.union(
                &{
                    let mut tm = TemporalMap::new();
                    tm.insert(&TemporalId::new(1_u64, t).unwrap(), 9);
                    tm
                },
                &ConflictPolicy::Overwrite,
            );
        }
        bb
    };

    let (sa, sb) = (secmap(&a), secmap(&b));

    // difference: A の時間から B の時間を除く（値は A）
    let d = a.difference(&b);
    assert_normalized(&d);
    let exp_d: BTreeMap<u64, i32> = sa
        .iter()
        .filter(|(k, _)| !sb.contains_key(k))
        .map(|(&k, &v)| (k, v))
        .collect();
    assert_eq!(secmap(&d), exp_d);

    // union（Overwrite=後勝ち=B優先）
    let u = a.union(&b, &ConflictPolicy::Overwrite);
    assert_normalized(&u);
    let mut exp_u = sa.clone();
    for (&k, &v) in &sb {
        exp_u.insert(k, v); // B で上書き
    }
    assert_eq!(secmap(&u), exp_u);

    // intersection（Overwrite=B優先）
    let i = a.intersection(&b, &ConflictPolicy::Overwrite);
    assert_normalized(&i);
    let exp_i: BTreeMap<u64, i32> = sb
        .iter()
        .filter(|(k, _)| sa.contains_key(k))
        .map(|(&k, &v)| (k, v))
        .collect();
    assert_eq!(secmap(&i), exp_i);

    // union（Max）
    let um = a.union(&b, &ConflictPolicy::Max);
    let mut exp_um = sa.clone();
    for (&k, &v) in &sb {
        let e = exp_um.entry(k).or_insert(v);
        *e = (*e).max(v);
    }
    assert_eq!(secmap(&um), exp_um);
}

/// get（二分探索）の照合。
#[test]
fn get_oracle() {
    let mut m = seg(60, 0, 1);
    m = m.union(&seg(60, 3, 2), &ConflictPolicy::Overwrite); // [0,60)=1, [180,240)=2
    let s = secmap(&m);
    for probe in [0u64, 30, 59, 60, 179, 180, 239, 240, 1000] {
        assert_eq!(m.contains_unixtime(probe), s.get(&probe), "get({probe})");
    }
}

/// cells 往復で被覆と値が保たれる。
#[test]
fn cells_roundtrip() {
    let mut m = seg(3600, 0, 7); // [0,3600)=7
    m = m.union(&seg(60, 60, 8), &ConflictPolicy::Overwrite); // [3600,3660)=8
    let mut rebuilt = TemporalMap::new();
    for (c, v) in m.iter() {
        rebuilt = rebuilt.union(
            &{
                let mut tm = TemporalMap::new();
                tm.insert(&c, *v);
                tm
            },
            &ConflictPolicy::Overwrite,
        );
    }
    assert_eq!(secmap(&m), secmap(&rebuilt));
}

#[test]
fn map_ergonomic_apis() {
    let mut m = TemporalMap::new();
    assert!(m.is_empty());
    assert_eq!(m.len(), 0);

    let t1 = TemporalId::new(3600_u64, 0_u64).unwrap();
    m.insert(&t1, 42);
    assert!(!m.is_empty());
    assert_eq!(m.len(), 1);
    assert_eq!(m.contains_unixtime(1800), Some(&42));
    assert_eq!(m.contains_unixtime(5000), None);

    assert!(m.iter().any(|(t, v)| t == t1 && *v == 42));
    assert!(m.temporal_ids().any(|t| t == t1));
    assert!(m.values().any(|v| *v == 42));

    let t2 = TemporalId::new(3600_u64, 1_u64).unwrap();
    m.insert(&t2, 100);
    assert_eq!(m.len(), 2);

    m.remove(&t1);
    assert_eq!(m.len(), 1);
    assert_eq!(m.contains_unixtime(1800), None);
    assert_eq!(m.contains_unixtime(5000), Some(&100));

    m.clear();
    assert!(m.is_empty());
    assert_eq!(m.len(), 0);
}
