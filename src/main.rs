use kasane_logic::{
    space_time_id::{SpaceTimeId, dimension::Dimension},
    space_time_id_set::SpaceTimeIdSet,
};

fn main() {
    let id = SpaceTimeId::new(
        4,
        Dimension {
            start: Some(-4),
            end: Some(5),
        },
        Dimension {
            start: Some(3),
            end: Some(6),
        },
        Dimension {
            start: Some(1),
            end: Some(10),
        },
        0,
        Dimension {
            start: None,
            end: None,
        },
    )
    .unwrap();

    println!("{}", id);

    let mut set = SpaceTimeIdSet::new();

    set.insert(id);
}
