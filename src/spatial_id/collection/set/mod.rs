use crate::spatial_id::collection::{Collection, set::logic::SetLogic};

pub mod logic;
pub mod memory;
#[cfg(test)]
mod tests;

pub trait SetStorage {}

impl<S: SetStorage + Collection> SetStorage for SetLogic<S> {}

impl<S: Collection + SetStorage> Collection for SetLogic<S> {
    type Main = S::Main;
    type Dimension = S::Dimension;

    fn main(&self) -> &Self::Main {
        self.0.main()
    }
    fn main_mut(&mut self) -> &mut Self::Main {
        self.0.main_mut()
    }

    fn f(&self) -> &Self::Dimension {
        self.0.f()
    }
    fn f_mut(&mut self) -> &mut Self::Dimension {
        self.0.f_mut()
    }

    fn x(&self) -> &Self::Dimension {
        self.0.x()
    }
    fn x_mut(&mut self) -> &mut Self::Dimension {
        self.0.x_mut()
    }

    fn y(&self) -> &Self::Dimension {
        self.0.y()
    }
    fn y_mut(&mut self) -> &mut Self::Dimension {
        self.0.y_mut()
    }

    fn fetch_flex_rank(&mut self) -> u64 {
        self.0.fetch_flex_rank()
    }
    fn return_flex_rank(&mut self, rank: u64) {
        self.0.return_flex_rank(rank)
    }

    fn move_flex_rank(&self) -> u64 {
        self.0.move_flex_rank()
    }

    fn move_flex_rank_free_list(&self) -> Vec<u64> {
        self.0.move_flex_rank_free_list()
    }
}
