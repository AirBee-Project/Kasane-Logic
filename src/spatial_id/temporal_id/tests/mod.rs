use crate::Interval;
use crate::SingleId;
use crate::TemporalId;
use crate::{FlexId, RangeId};
use alloc::string::ToString;
use core::str::FromStr;
pub mod domain;

#[test]
/// temporal_idが正しく作成できることを検証する
fn create_temporal_id_success() {
    let temporal_id = TemporalId::new(60_u64, 10).unwrap();
    let single_id = SingleId::new(10, 10, 10, 10)
        .unwrap()
        .with_temporal(temporal_id);

    assert_eq!(single_id.to_string(), "10/10/10/10_60/10")
}

#[test]
/// DisplayとFromStrが正しいか
fn test_temporal_id_display_fromstr_roundtrip() {
    let t = TemporalId::new(3600_u64, 10).unwrap();
    let s = t.to_string();
    let parsed = TemporalId::from_str(&s).unwrap();
    assert_eq!(t, parsed);
}

#[test]
/// DisplayとFromStrが正しいか
fn test_single_id_display_fromstr_roundtrip() {
    let mut id = SingleId::new(4, 6, 9, 10).unwrap();

    id = id.with_temporal(TemporalId::new(3600_u64, 10).unwrap());

    let s = id.to_string();
    let parsed = SingleId::from_str(&s).unwrap();
    assert_eq!(id, parsed);
}

#[test]
/// DisplayとFromStrが正しいか
fn test_range_id_display_fromstr_roundtrip() {
    let mut id = RangeId::new(4, [6, 6], [9, 9], [10, 10]).unwrap();
    id = id.with_temporal(TemporalId::new(3600_u64, 10).unwrap());

    let s = id.to_string();
    let parsed = RangeId::from_str(&s).unwrap();
    assert_eq!(id, parsed);
}

#[test]
fn test_flex_id_display_fromstr_roundtrip() {
    let mut id = FlexId::new(4, 6, 2, 3, 5, 2).unwrap();
    id = id.with_temporal(TemporalId::new(3600_u64, 10).unwrap());

    let s = id.to_string();
    let parsed = FlexId::from_str(&s).unwrap();
    assert_eq!(id, parsed);
}

#[test]
/// temporal_idの作成が正しく失敗することを示す
fn create_temporal_id_failure() {
    // i=30は無理
    let temporal_id = TemporalId::new(30_u64, 10);
    assert!(temporal_id.is_err());

    // i=0もエラーになるはず
    let temporal_id = TemporalId::new(0_u64, 10);
    assert!(temporal_id.is_err());
}

#[test]
/// WHOLEの時の挙動を見る
fn create_whole_temporal_id_failure() {
    // WHOLEの10なんてないので、失敗するはず
    let temporal_id = TemporalId::new(Interval::Whole, 10);
    assert!(temporal_id.is_err())
}

#[test]
/// WHOLEの時の挙動を見る
fn create_whole_temporal_id() {
    let temporal_id = TemporalId::new(Interval::Whole, 0).unwrap();

    let single_id = SingleId::new(10, 10, 10, 10)
        .unwrap()
        .with_temporal(temporal_id);

    // WHOLEの場合は時間は無限となり、時間ID部分が表示されない
    assert_eq!(single_id.to_string(), "10/10/10/10")
}
