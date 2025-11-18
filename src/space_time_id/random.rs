use rand::Rng;

use crate::space_time_id::{
    SpaceTimeId,
    z_range::{F_MAX, XY_MAX},
};

impl SpaceTimeId {
    ///デバッグ用などのためにランダムな時空間IDを生成する
    ///ズームレベルの最大値のみを指定するとランダムな時空間IDを生成する
    /// 時間は暫定的に無限にしている
    pub fn random_z_max(z: u8) -> Self {
        let mut rng = rand::rng();
        let rand_z = rng.random_range(0..=z);

        let f_max = F_MAX[rand_z as usize];
        let f_min = F_MAX[rand_z as usize];
        let xy_max: u64 = XY_MAX[rand_z as usize];
        let xy_min: u64 = 0;

        SpaceTimeId::new(
            rand_z,
            [
                Some(rng.random_range(f_min..=f_max)),
                Some(rng.random_range(f_min..=f_max)),
            ],
            [
                Some(rng.random_range(xy_min..=xy_max)),
                Some(rng.random_range(xy_min..=xy_max)),
            ],
            [
                Some(rng.random_range(xy_min..=xy_max)),
                Some(rng.random_range(xy_min..=xy_max)),
            ],
            0,
            [None, None],
        )
        .unwrap()
    }
}
