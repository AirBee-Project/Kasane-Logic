#[macro_export]
macro_rules! range_opt {
    // [-] → [None, None]
    ([-]) => {
        [None, None]
    };

    ([-,-]) => {
        [None, None]
    };

    // [x] → [Some(x), Some(x)]
    ([$x:expr]) => {
        [Some($x), Some($x)]
    };

    // [-, b] → [None, Some(b)]
    ([-, $b:expr]) => {
        [None, Some($b)]
    };

    // [a, -] → [Some(a), None]
    ([$a:expr, -]) => {
        [Some($a), None]
    };

    // [a, b] → [Some(a), Some(b)] (a ≤ b)
    ([$a:expr, $b:expr]) => {{ [Some($a), Some($b)] }};
}

///時空間IDを生成するマクロ
///
/// 厳密な拡張記法に対応
#[macro_export]
macro_rules! id {
    // 時空間が指定されているパターン
    (
        z: $z:expr,
        f: $f:tt,
        x: $x:tt,
        y: $y:tt,
        t: $t:tt $(,)?
    ) => {{
        let f_range: [Option<i64>; 2] = $crate::range_opt!($f);
        let x_range: [Option<u64>; 2] = $crate::range_opt!($x);
        let y_range: [Option<u64>; 2] = $crate::range_opt!($y);
        let t_range: [Option<u64>; 2] = $crate::range_opt!($t);

        SpaceTimeID::new($z, f_range, x_range, y_range, t_range)
    }};
}
