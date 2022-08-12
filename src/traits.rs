use crate::error::IonResult;
use crate::walker::IonWalker;

pub trait IonDeserialize: Sized {
    fn deserialize<'d>(walker: &IonWalker<'d>) -> IonResult<Self>;
}
