use crate::{
    SingleId, Source, SpatialIdTable,
    spatial_id::collection::query::execution::Query,
    spatial_id::collection::query::merge_policy::{Average, Max, Min, Sum},
    spatial_id::collection::query::traits::WorkingTree,
};
use alloc::vec::Vec;
use proptest::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Policy {
    Max,
    Min,
    Sum,
    Average,
}

fn arb_policy() -> impl Strategy<Value = Policy> {
    prop_oneof![
        Just(Policy::Max),
        Just(Policy::Min),
        Just(Policy::Sum),
        Just(Policy::Average),
    ]
}

macro_rules! dispatch_policy {
    ($q:expr, $method:ident ($($args:expr),*), $policy:expr) => {
        match $policy {
            Policy::Max => $q.$method($($args,)* Max),
            Policy::Min => $q.$method($($args,)* Min),
            Policy::Sum => $q.$method($($args,)* Sum),
            Policy::Average => $q.$method($($args,)* Average),
        }
    };
}

macro_rules! define_query_ops {
    ($( $variant:ident ($($arg_ty:ty),*) => $gen:expr => |$q:ident, $($parg:ident),*| $apply:expr ),* $(,)?) => {
        #[derive(Debug, Clone)]
        enum QueryOp {
            $( $variant($($arg_ty),*) ),*
        }

        fn arb_op(z: u8) -> impl Strategy<Value = QueryOp> {
            let zl = crate::ZoomLevel::new(z).unwrap();
            let f_min = zl.f_min();
            let f_max = zl.f_max();
            let xy_max = zl.xy_max();

            prop_oneof![
                $(
                    $gen(z, zl, f_min, f_max, xy_max)
                ),*
            ]
        }

        impl QueryOp {
            fn apply<W: WorkingTree<Value = u32> + 'static>(&self, q: Query<W>) -> Query<W> {
                match self {
                    $(
                        QueryOp::$variant($($parg),*) => {
                            let $q = q;
                            $apply
                        }
                    ),*
                }
            }
        }
    };
}

define_query_ops! {
    ShiftX(u8, i32)
        => |z, _zl, _fmin, _fmax, _xymax| (-5..=5i32).prop_map(move |off| QueryOp::ShiftX(z, off))
        => |q, z, offset| q.shift_x(*z, *offset),

    ShiftY(u8, i32)
        => |z, _zl, _fmin, _fmax, _xymax| (-5..=5i32).prop_map(move |off| QueryOp::ShiftY(z, off))
        => |q, z, offset| q.shift_y(*z, *offset),

    ShiftF(u8, i32)
        => |z, _zl, _fmin, _fmax, _xymax| (-5..=5i32).prop_map(move |off| QueryOp::ShiftF(z, off))
        => |q, z, offset| q.shift_f(*z, *offset),

    FalloffLinearX(u8, u32, Policy)
        => |z, _zl, _fmin, _fmax, _xymax| (1..=2u32, arb_policy()).prop_map(move |(r, p)| QueryOp::FalloffLinearX(z, r, p))
        => |q, z, r, p| dispatch_policy!(q, falloff_linear_x(*z, *r), p),

    FalloffLinearY(u8, u32, Policy)
        => |z, _zl, _fmin, _fmax, _xymax| (1..=2u32, arb_policy()).prop_map(move |(r, p)| QueryOp::FalloffLinearY(z, r, p))
        => |q, z, r, p| dispatch_policy!(q, falloff_linear_y(*z, *r), p),

    FalloffLinearF(u8, u32, Policy)
        => |z, _zl, _fmin, _fmax, _xymax| (1..=2u32, arb_policy()).prop_map(move |(r, p)| QueryOp::FalloffLinearF(z, r, p))
        => |q, z, r, p| dispatch_policy!(q, falloff_linear_f(*z, *r), p),

    ExtrudeX(u8, u32, u32, Policy)
        => |z, _zl, _fmin, _fmax, xymax: u32| (0..=xymax, 0..=5u32, arb_policy()).prop_map(move |(s, l, p)| QueryOp::ExtrudeX(z, s, u32::min(s + l, xymax), p))
        => |q, z, start, end, p| dispatch_policy!(q, extrude_x(*z, *start, *end), p),

    ExtrudeY(u8, u32, u32, Policy)
        => |z, _zl, _fmin, _fmax, xymax: u32| (0..=xymax, 0..=5u32, arb_policy()).prop_map(move |(s, l, p)| QueryOp::ExtrudeY(z, s, u32::min(s + l, xymax), p))
        => |q, z, start, end, p| dispatch_policy!(q, extrude_y(*z, *start, *end), p),

    ExtrudeF(u8, i32, i32, Policy)
        => |z, _zl, fmin: i32, fmax: i32, _xymax| (fmin..=fmax, 0..=5i32, arb_policy()).prop_map(move |(s, l, p)| QueryOp::ExtrudeF(z, s, i32::min(s + l, fmax), p))
        => |q, z, start, end, p| dispatch_policy!(q, extrude_f(*z, *start, *end), p),
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 20,
        .. ProptestConfig::default()
    })]

    #[test]
    #[ignore]
    fn test_random_query_raw_run_matches_run(
        z in 10..=12u8,
        items in prop::collection::vec((SingleId::arb_within(10..=12), 1..100u32), 1..5),
        ops in prop::collection::vec(10..=12u8, 1..3).prop_flat_map(|zs| {
            zs.into_iter().map(arb_op).collect::<Vec<_>>()
        })
    ) {
        let mut table = SpatialIdTable::new();
        for (id, val) in items {
            if let Ok(adjusted_id) = SingleId::new(z, id.f(), id.x(), id.y()) {
                table.insert(adjusted_id, val);
            }
        }

        if table.is_empty() {
            return Ok(());
        }

        let mut q_raw = table.clone().query();
        let mut q_run = table.clone().query();

        for op in &ops {
            q_raw = op.apply(q_raw);
            q_run = op.apply(q_run);
        }

        let res_raw: Result<SpatialIdTable<u32>, _> = q_raw.raw_run();
        let res_run: Result<SpatialIdTable<u32>, _> = q_run.run();

        match (res_raw, res_run) {
            (Ok(raw), Ok(run)) => {
                assert_eq!(
                    raw.flat_single_ids().collect::<Vec<_>>(),
                    run.flat_single_ids().collect::<Vec<_>>(),
                    "raw_run and run produced different ID sets!"
                );
            }
            (Err(_), Err(_)) => {
                // 両方エラーならOK
            }
            (Ok(_), Err(e)) => {
                panic!("raw_run succeeded but run failed with {:?}", e);
            }
            (Err(e), Ok(_)) => {
                panic!("run succeeded but raw_run failed with {:?}", e);
            }
        }
    }
}
