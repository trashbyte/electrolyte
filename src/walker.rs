use chrono::{DateTime, FixedOffset};
use crate::error::{IonError, IonErrorType, IonResult};
use crate::types::{IonStruct, IonType, IonValue, IonList, Annotations};
use paste::paste;
use crate::traits::IonDeserialize;

macro_rules! type_fns {
    ($ion_ty:ident, $pat:pat => $res:expr; $ret:ty) => {
        paste! {
            #[doc = "Attempt to read the current value as a " $ion_ty]
            pub fn [< as_ $ion_ty:lower >](&self) -> IonResult<$ret> {
                match &self.data {
                    IonValue::$ion_ty$pat => $res,
                    _ => Err(IonError::new(IonErrorType::WrongType { found: self.data.ty(), expected: IonType::$ion_ty }, self.clone_scopes()))
                }
            }

            #[doc = "Attempt to read the named field as a " $ion_ty ". Assumes current value is an `IonStruct`."]
            pub fn [< get_ $ion_ty:lower >](&self, field_name: impl AsRef<str>) -> IonResult<$ret> {
                match self.as_struct()?.field(field_name.as_ref()) {
                    Some(val) => {
                        match val {
                            IonValue::$ion_ty$pat => $res,
                            _ => Err(IonError::new(
                                IonErrorType::WrongType { found: val.ty(), expected: IonType::$ion_ty },
                                self.clone_scopes_with(field_name)
                            )),
                        }
                    },
                    None => Err(IonError::new(
                        IonErrorType::MissingField(field_name.as_ref().to_string()),
                        self.clone_scopes_with(field_name)
                    ))
                }
            }
        }
    }
}

pub struct IonWalker<'d> {
    data: &'d IonValue,
    scopes: Vec<String>,
}
impl<'d> IonWalker<'d> {
    /// Construct an IonWalker around the given reference, with no scopes.
    pub fn new(data: &'d IonValue) -> Self {
        IonWalker { data, scopes: Vec::new() }
    }
    /// Construct an IonWalker around the given reference, with the given scopes.
    pub fn with_scopes(data: &'d IonValue, scopes: Vec<String>) -> Self {
        IonWalker { data, scopes }
    }

    /// Returns a copy of this IonWalker with an added scope
    pub fn clone_with_scope(&self, scope: impl AsRef<str>) -> Self {
        let mut scopes = self.scopes.clone();
        scopes.push(scope.as_ref().to_string());
        IonWalker { data: self.data, scopes }
    }

    /// Returns a copy of this IonWalker's scopes
    pub fn clone_scopes(&self) -> Vec<String> { self.scopes.clone() }

    /// Returns a copy of this IonWalker's scopes with an extra scope added
    pub fn clone_scopes_with(&self, scope: impl AsRef<str>) -> Vec<String> {
        let mut scopes = self.scopes.clone();
        scopes.push(scope.as_ref().to_string());
        scopes
    }

    /// Convenience function to create an IonError with a copy of this IonWalker's scopes
    pub fn error(&self, error: IonErrorType) -> IonError {
        IonError::new(error, self.clone_scopes())
    }

    /// Convenience function for deserializing values that are IonDeserialize
    pub fn deserialize<T: IonDeserialize>(data: &IonValue) -> IonResult<T> {
        T::deserialize(&mut IonWalker::new(data))
    }

    /// Convenience function for deserializing values that are IonDeserialize
    pub fn deserialize_with_scopes<T: IonDeserialize>(data: &IonValue, scopes: &Vec<String>) -> IonResult<T> {
        T::deserialize(&mut IonWalker::with_scopes(data, scopes.clone()))
    }

    /// Returns the list of annotations for the current value.
    pub fn annotations(&self) -> &Annotations {
        self.data.annotations()
    }

    /// Returns true if the current value has an annotation with the given value.
    pub fn has_annotation(&self, ann: impl AsRef<str>) -> bool {
        self.data.annotations().iter().find(|a| a.as_str() == ann.as_ref()).is_some()
    }

    type_fns!(Struct,   (s,_) => Ok(s);         &IonStruct);
    type_fns!(List,     (l,_) => Ok(l);         &IonList);
    type_fns!(Null,     (_)   => Ok(());        ());
    type_fns!(Boolean,  (b,_) => Ok(*b);        bool);
    type_fns!(Integer,  (i,_) => Ok(*i);        i64);
    type_fns!(Float,    (f,_) => Ok(*f);        f64);
    type_fns!(String,   (s,_) => Ok(s);         &str);
    type_fns!(Blob,     (b,_) => Ok(&b[..]);    &[u8]);
    type_fns!(Timestamp,(t,_) => Ok(t);         &DateTime<FixedOffset>);

    /// Generic version of the as_X method that works for any type which is `IonDeserialize`.
    pub fn as_type<T: IonDeserialize>(&self) -> IonResult<T> {
        T::deserialize(&self)
    }

    /// Generic version of the get_X method that works for any type which is `IonDeserialize`.
    pub fn get_type<T: IonDeserialize>(&self, field_name: impl AsRef<str>) -> IonResult<T> {
        match self.as_struct()?.field(field_name.as_ref()) {
            Some(field) => {
                T::deserialize(&IonWalker {
                    data: field,
                    scopes: self.clone_scopes_with(field_name)
                })
            }
            None => Err(IonError::new(
                IonErrorType::MissingField(field_name.as_ref().to_string()),
                self.clone_scopes_with(field_name)
            ))
        }
    }
}
