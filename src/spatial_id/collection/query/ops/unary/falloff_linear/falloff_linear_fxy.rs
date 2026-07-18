use alloc::boxed::Box;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::{Div, Mul, Sub};

use crate::{
    Error, SpatialIdCollection, ZoomLevel,
    spatial_id::collection::query::{merge_policy::MergePolicy, traits::UnaryOperator},
};

pub struct FalloffLinearFxy<P> {
    z: ZoomLevel,
    f_radius: u32,
    x_radius: u32,
    y_radius: u32,
    _marker: PhantomData<P>,
}

impl<P> FalloffLinearFxy<P> {
    pub fn new<T: Into<u8>>(
        z: T,
        f_radius: u32,
        x_radius: u32,
        y_radius: u32,
    ) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self {
            z,
            f_radius,
            x_radius,
            y_radius,
            _marker: PhantomData,
        })
    }
}

impl<S, P> UnaryOperator<S> for FalloffLinearFxy<P>
where
    S: SpatialIdCollection,
    P: MergePolicy<S::Value> + Send + Sync,
    S::Value: Mul<Output = S::Value>
        + Div<Output = S::Value>
        + Sub<Output = S::Value>
        + TryFrom<u32>
        + Clone
        + Send
        + Sync,
    <S::Value as TryFrom<u32>>::Error: Debug,
{
    fn run(&self, target: &mut S) -> Result<(), Box<dyn core::error::Error + 'static>> {
        let z = self.z.get();
        let f_rad = self.f_radius as i32;
        let x_rad = self.x_radius as i32;
        let y_rad = self.y_radius as i32;

        if f_rad == 0 && x_rad == 0 && y_rad == 0 {
            return Ok(());
        }

        let snapshot: alloc::vec::Vec<_> = target.iter().map(|(id, v)| (id, v.clone())).collect();

        #[cfg(feature = "rayon")]
        let mut mapped: alloc::vec::Vec<_> = {
            use rayon::prelude::*;
            snapshot
                .into_par_iter()
                .map(|(id, value)| {
                    let mut cells = alloc::vec::Vec::new();
                    if let Ok(iter) = id.falloff_linear_fxy(
                        z,
                        self.f_radius,
                        self.x_radius,
                        self.y_radius,
                        &value,
                    ) {
                        cells.extend(iter);
                    }
                    cells
                })
                .flatten()
                .collect()
        };
        #[cfg(not(feature = "rayon"))]
        let mut mapped: alloc::vec::Vec<_> = {
            let mut out = alloc::vec::Vec::new();
            for (id, value) in snapshot {
                if let Ok(iter) =
                    id.falloff_linear_fxy(z, self.f_radius, self.x_radius, self.y_radius, &value)
                {
                    out.extend(iter);
                }
            }
            out
        };

        #[cfg(feature = "rayon")]
        {
            use rayon::prelude::*;
            mapped.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
        }
        #[cfg(not(feature = "rayon"))]
        mapped.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut merged = alloc::vec::Vec::new();
        let mut current = None;

        for (id, value) in mapped {
            match current {
                Some((curr_id, curr_val)) if curr_id == id => {
                    let resolved = P::resolve(curr_val, value);
                    current = Some((curr_id, resolved));
                }
                Some((curr_id, curr_val)) => {
                    merged.push((curr_id, curr_val));
                    current = Some((id, value));
                }
                None => {
                    current = Some((id, value));
                }
            }
        }
        if let Some((curr_id, curr_val)) = current {
            merged.push((curr_id, curr_val));
        }

        let mut new_target = S::from_items(alloc::vec::Vec::new());
        crate::spatial_id::collection::query::utils::insert_with_policy::<S, P>(
            &mut new_target,
            merged,
        )?;

        *target = new_target;
        Ok(())
    }
}
