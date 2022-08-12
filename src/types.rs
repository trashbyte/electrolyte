use std::collections::HashMap;
use std::ops::Index;
use chrono::{DateTime, FixedOffset};
use crate::{IonDeserialize, IonWalker, IonResult};

pub type Annotations = Vec<String>;

#[derive(Debug, Clone, PartialEq)]
/// A value in an Ion data structure with any annotations
pub enum IonValue {
    Null(Annotations),
    Boolean(bool, Annotations),
    Integer(i64, Annotations),
    Float(f64, Annotations),
    Timestamp(DateTime<FixedOffset>, Annotations),
    Blob(Vec<u8>, Annotations),
    String(String, Annotations),
    List(IonList, Annotations),
    Struct(IonStruct, Annotations),
}
impl IonValue {
    /// Returns the IonType of this value
    pub fn ty(&self) -> IonType {
        match &self {
            IonValue::Null(_) => IonType::Null,
            IonValue::Boolean(_,_) => IonType::Boolean,
            IonValue::Integer(_,_) => IonType::Integer,
            IonValue::Float(_,_) => IonType::Float,
            IonValue::Timestamp(_,_) => IonType::Timestamp,
            IonValue::String(_,_) => IonType::String,
            IonValue::Blob(_,_) => IonType::Blob,
            IonValue::List(_,_) => IonType::List,
            IonValue::Struct(_,_) => IonType::Struct,
        }
    }

    /// Attempts to retrieve a reference to an `IonStruct`.
    /// Returns `None` if the value is of a different type.
    pub fn as_struct(&self) -> Option<&IonStruct> {
        if let IonValue::Struct(st,_) = &self { Some(st) }
        else { None }
    }

    /// Attempts to retrieve a reference to an `IonList`.
    /// Returns `None` if the value is of a different type.
    pub fn as_list(&self) -> Option<&IonList> {
        if let IonValue::List(list,_) = &self { Some(list) }
        else { None }
    }
    /// Attempts to retrieve a reference to a sized list of `IonValue`s.
    /// Returns `None` if the value is of a different type or a list of a different size.
    pub fn as_list_sized<const N: usize>(&self) -> Option<&[IonValue; N]> {
        if let IonValue::List(list,_) = &self {
            if list.len() == N {
                return Some((&list.items[0..N]).try_into().unwrap());
            }
        }
        None
    }
    /// Attempts to retrieve a boolean value. Returns `None` if the value is of a different type.
    pub fn as_bool(&self) -> Option<bool> {
        if let IonValue::Boolean(b,_) = &self { Some(*b) }
        else { None }
    }
    /// Attempts to retrieve an integer value. Returns `None` if the value is of a different type.
    pub fn as_int(&self) -> Option<i64> {
        if let IonValue::Integer(i,_) = &self { Some(*i) }
        else { None }
    }
    /// Attempts to retrieve a float value. Returns `None` if the value is of a different type.
    pub fn as_float(&self) -> Option<f64> {
        match &self {
            IonValue::Float(f,_) => Some(*f),
            IonValue::Integer(i,_) => Some(*i as f64),
            _ => None
        }
    }
    /// Attempts to retrieve a string value. Returns `None` if the value is of a different type.
    pub fn as_str(&self) -> Option<&str> {
        if let IonValue::String(s,_) = &self { Some(s.as_ref()) }
        else { None }
    }
    /// Attempts to retrieve a timestamp value. Returns `None` if the value is of a different type.
    pub fn as_timestamp(&self) -> Option<&DateTime<FixedOffset>> {
        if let IonValue::Timestamp(ts,_) = &self { Some(ts) }
        else { None }
    }
    /// Checks if the value is of the given type.
    pub fn is(&self, ty: IonType) -> bool {
        self.ty() == ty
    }
    /// Returns a reference to this value's annotations.
    pub fn annotations(&self) -> &Annotations {
        match &self {
            IonValue::Null(ann) => ann,
            IonValue::Boolean(_,ann) => ann,
            IonValue::Integer(_,ann) => ann,
            IonValue::Float(_,ann) => ann,
            IonValue::Timestamp(_,ann) => ann,
            IonValue::Blob(_,ann) => ann,
            IonValue::String(_,ann) => ann,
            IonValue::List(_,ann) => ann,
            IonValue::Struct(_,ann) => ann,
        }
    }
    /// Returns true if the value has any annotations.
    pub fn has_annotation(&self, annotation: impl AsRef<str>) -> bool {
        self.annotations().iter()
                          .find(|ann| ann.as_str() == annotation.as_ref())
                          .is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// An enum of all supported Ion types. `IonType` does not contain any data, `IonValue` does.
pub enum IonType {
    Null,
    Boolean,
    Integer,
    Float,
    Timestamp,
    String,
    Blob,
    List,
    Struct,
}

#[derive(Debug, Clone, PartialEq)]
/// An Ion struct. Thin wrapper over a map of strings to `IonValue`s.
pub struct IonStruct {
    fields: HashMap<String, IonValue>,
}

impl IonStruct {
    /// Create a new `IonStruct` from the given field map.
    pub fn new(fields: HashMap<String, IonValue>) -> Self { IonStruct { fields } }
    /// Create a new `IonStruct` with no fields.
    pub fn new_empty() -> Self { IonStruct { fields: HashMap::new() } }

    /// Attempts to retrieve a field with the given name. Returns `None` if the field is not present.
    pub fn field(&self, name: &str) -> Option<&IonValue> {
        self.fields.get(name)
    }

    /// Returns an iterator over the struct's fields.
    pub fn iter_fields(&self) -> std::collections::hash_map::Iter<String, IonValue> {
        self.fields.iter()
    }

    /// Attempt to convert all fields to the given `IonDeserialize` type.
    /// Fails on deserialization error or if any of the field values are not `T`.
    pub fn into_map_of<T: IonDeserialize>(&self, scopes: Option<Vec<String>>)
        -> IonResult<HashMap<String, T>>
    {
        let mut map = HashMap::new();
        let scopes = scopes.unwrap_or(Vec::new());
        for (k, v) in self.fields.iter() {
            let value = IonWalker::deserialize_with_scopes(v, &scopes)?;
            map.insert(k.clone(), value);
        }
        Ok(map)
    }
}

#[derive(Debug, Clone, PartialEq)]
/// An Ion list. Thin wrapper over a vec of `IonValue`s.
pub struct IonList {
    pub items: Vec<IonValue>,
}
impl IonList {
    /// Returns an iterator over the list's `IonValue`s.
    pub fn iter(&self) -> std::slice::Iter<IonValue> { self.items.iter() }
    /// Returns the length of the underlying list.
    pub fn len(&self) -> usize { self.items.len() }
    /// Returns the `IonValue` at the given index if it is in bounds, `None` otherwise.
    /// Use square bracket indexing (`Index<usize>`) for unchecked access.
    pub fn at(&self, idx: usize) -> Option<&IonValue> { self.items.get(idx) }
}
impl Index<usize> for IonList {
    type Output = IonValue;
    fn index(&self, index: usize) -> &IonValue { &self.items[index] }
}
