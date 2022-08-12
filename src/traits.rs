use crate::error::IonResult;
use crate::walker::IonWalker;

pub trait IonDeserialize: Sized {
    fn deserialize<'d>(walker: &IonWalker<'d>) -> IonResult<Self>;
}

impl IonDeserialize for f32 {
    fn deserialize(walker: &IonWalker) -> IonResult<Self> {
        Ok(walker.as_float()? as f32)
    }
}
impl IonDeserialize for f64 {
    fn deserialize(walker: &IonWalker) -> IonResult<Self> {
        walker.as_float()
    }
}
impl IonDeserialize for i32 {
    fn deserialize(walker: &IonWalker) -> IonResult<Self> {
        Ok(walker.as_integer()? as i32)
    }
}
impl IonDeserialize for i64 {
    fn deserialize(walker: &IonWalker) -> IonResult<Self> {
        walker.as_integer()
    }
}