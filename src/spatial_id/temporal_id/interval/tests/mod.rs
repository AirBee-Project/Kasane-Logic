use super::Interval;
use crate::TemporalId;

#[test]
fn seconds_roundtrip() {
    for iv in Interval::coarse_to_fine() {
        assert_eq!(Interval::new(iv.seconds()).unwrap(), iv, "{iv:?}");
    }
    // u64::MAX は約数鎖に無い（Whole の別名扱いはしない）
    assert!(Interval::new(u64::MAX).is_err());
    assert!(Interval::new(7200).is_err()); // 約数鎖に無い
    assert!(Interval::new(86400 * 3).is_err()); // 2冪でない日倍数
    assert!(Interval::new(0).is_err());
}

#[test]
fn try_from_numeric() {
    // try_from for u64
    assert_eq!(Interval::try_from(3600u64).unwrap(), Interval::Hour);
    assert_eq!(
        Interval::try_from(86400 * 4u64).unwrap(),
        Interval::day_pow(2).unwrap()
    );
    assert!(Interval::try_from(7200u64).is_err());

    // try_from for i32
    assert_eq!(Interval::try_from(3600i32).unwrap(), Interval::Hour);
    assert!(Interval::try_from(-3600i32).is_err());
}

#[test]
fn day_pow_validated() {
    assert_eq!(Interval::day_pow(0).unwrap(), Interval::Day);
    assert_eq!(Interval::day_pow(1).unwrap().seconds(), 86400 * 2);
    assert_eq!(Interval::day_pow(47).unwrap(), Interval::Whole);
    // 範囲外は構築できない（不正な指数を持つ Interval は存在しない）
    assert!(Interval::day_pow(48).is_err());
    assert!(Interval::day_pow(u8::MAX).is_err());
    // 読み取りはパターンマッチできる
    match Interval::day_pow(3).unwrap() {
        Interval::DayPow { k, .. } => assert_eq!(k, 3),
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn chain_is_divisor_chain() {
    // 隣接する段は必ず割り切れる（入れ子・非交差の根拠）。
    let chain: alloc::vec::Vec<Interval> = Interval::coarse_to_fine().collect();
    for w in chain.windows(2) {
        let (coarse, fine) = (w[0].seconds(), w[1].seconds());
        assert!(
            coarse % fine == 0 && coarse > fine,
            "not a divisor chain: {:?} -> {:?}",
            w[0],
            w[1]
        );
    }
    // 最上段はドメイン全体
    assert_eq!(chain[0], Interval::Whole);
    assert_eq!(Interval::Whole.seconds(), Interval::WHOLE_SECONDS);
}

#[test]
fn ordering_coarse_to_fine() {
    assert!(Interval::Whole < Interval::day_pow(46).unwrap());
    assert!(Interval::day_pow(46).unwrap() < Interval::day_pow(1).unwrap());
    assert!(Interval::day_pow(1).unwrap() < Interval::Day);
    assert!(Interval::Whole < Interval::Day);
    assert!(Interval::Day < Interval::Hour);
    assert!(Interval::Hour < Interval::Minute);
    assert!(Interval::Minute < Interval::Second);
}

#[test]
fn temporal_id_interval_accessor() {
    let id = TemporalId::new(Interval::Hour, 10).unwrap();
    assert_eq!(id.interval(), Interval::Hour);
    assert_eq!(id.i(), Interval::Hour);
    assert_eq!(id.start_unixtime(), 36000);
    // 生秒数の from_seconds とも一致
    assert_eq!(id, TemporalId::from_seconds(3600, 10).unwrap());
    // 二進層も from_seconds で構築できる
    let two_days = TemporalId::from_seconds(86400 * 2, 3).unwrap();
    assert_eq!(two_days.interval(), Interval::day_pow(1).unwrap());
    assert_eq!(two_days.start_unixtime(), 86400 * 6);
}
