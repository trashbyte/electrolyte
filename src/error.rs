use std::io::Error;
use ion_c_sys::result::IonCError;
use crate::types::IonType;

pub type IonResult<T> = Result<T, IonError>;

#[derive(Debug)]
pub struct IonError {
    pub ty: IonErrorType,
    pub scopes: Vec<String>,
}
impl IonError {
    pub fn new(ty: IonErrorType, scopes: Vec<String>) -> Self {
        IonError { ty, scopes }
    }
}

#[derive(Debug)]
pub enum IonErrorType {
    InvalidValue(String),
    MissingField(String),
    WrongType { found: IonType, expected: IonType },
    WrongSize { found: usize, expected: usize },
    IoError(std::io::Error),
    ParseError(IonCError),
    TypeNotSupported(&'static str),
    MissingAnnotation { expected: &'static [&'static str] },
    IndexOutOfBounds { tried: usize, bounds: (usize, usize) },
}

impl From<std::io::Error> for IonError {
    fn from(e: Error) -> Self {
        IonError::new(IonErrorType::IoError(e), Vec::new())
    }
}

impl From<IonCError> for IonError {
    fn from(e: IonCError) -> Self {
        IonError::new(IonErrorType::ParseError(e), Vec::new())
    }
}