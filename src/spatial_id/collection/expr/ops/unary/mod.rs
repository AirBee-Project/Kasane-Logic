use crate::UnaryOperator;

#[allow(dead_code)]
pub struct ShiftX {
    z: u8,
    x: u32,
}

impl UnaryOperator for ShiftX {
    type Parameter = ShiftX;

    fn new(parameter: Self::Parameter) -> Self {
        parameter
    }

    fn run<T: crate::SpatialIdCollection>(
        &self,
        _target: &mut T,
    ) -> Result<(), Box<dyn std::error::Error + 'static>> {
        Ok(())
    }
}
