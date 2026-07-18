use crate::UnaryOperator;

pub struct ShiftX {
    z: u8,
    x: u32,
}

impl UnaryOperator for ShiftX {
    type Parameter = ShiftX;

    fn run<T: crate::SpatialIdCollection>(
        &self,
        _target: &mut T,
    ) -> Result<(), Box<dyn std::error::Error + 'static>> {
        Ok(())
    }
}
