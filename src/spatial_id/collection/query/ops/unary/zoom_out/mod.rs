pub mod query;
#[allow(clippy::module_inception)]
pub mod zoom_out;

#[cfg(test)]
mod test;

pub use zoom_out::ZoomOut;
