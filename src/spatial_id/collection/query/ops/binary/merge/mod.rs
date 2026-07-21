#[allow(clippy::module_inception)]
pub mod merge;
pub mod query;

#[cfg(test)]
mod test;

pub use merge::Merge;
