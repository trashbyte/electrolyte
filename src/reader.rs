use std::collections::HashMap;
use std::path::Path;
use ion_c_sys::*;
use ion_c_sys::reader::{IonCReader, IonCReaderHandle};
use num_bigint::Sign;
use crate::error::{IonError, IonErrorType, IonResult};
use crate::types::{IonList, IonStruct, IonValue};

pub struct IonReader;
impl IonReader {
    pub fn read_file(path: impl AsRef<Path>) -> IonResult<IonValue> {
        let s = std::fs::read_to_string(path.as_ref())?;
        IonReader::read_string(&s)
    }

    pub fn read_string(string: &str) -> IonResult<IonValue> {
        let mut items = Vec::new();
        let mut reader = IonCReaderHandle::try_from(string)?;
        loop {
            let ty = reader.next()?;
            if ty == ION_TYPE_NONE || ty == ION_TYPE_EOF {
                return Ok(IonValue::List(IonList { items }, Vec::new()));
            }
            else {
                items.push(IonReader::read_value(&mut reader)?);
            }
        }
    }

    fn read_value(reader: &mut IonCReaderHandle) -> IonResult<IonValue> {
        let annotations = reader.get_annotations()?
            .iter()
            .map(|ann| ann.to_string())
            .collect();
        match reader.get_type()? {
            ION_TYPE_NULL => Ok(IonValue::Null(annotations)),
            ION_TYPE_SEXP => Err(IonError::new(IonErrorType::TypeNotSupported("SExpr"), Vec::new())),
            ION_TYPE_BLOB => Err(IonError::new(IonErrorType::TypeNotSupported("Blob"), Vec::new())),
            ION_TYPE_CLOB => Err(IonError::new(IonErrorType::TypeNotSupported("Clob"), Vec::new())),
            ION_TYPE_SYMBOL => Err(IonError::new(IonErrorType::TypeNotSupported("Symbol"), Vec::new())),
            ION_TYPE_DATAGRAM => Err(IonError::new(IonErrorType::TypeNotSupported("Datagram"), Vec::new())),
            ION_TYPE_STRUCT => Ok(IonValue::Struct(IonReader::read_struct(reader)?, annotations)),
            ION_TYPE_LIST => Ok(IonValue::List(IonReader::read_list(reader)?, annotations)),
            ION_TYPE_STRING => Ok(IonValue::String(reader.read_string()?.as_str().to_string(), annotations)),
            ION_TYPE_INT => Ok(IonValue::Integer(reader.read_i64()?, annotations)),
            ION_TYPE_FLOAT => Ok(IonValue::Float(reader.read_f64()?, annotations)),
            ION_TYPE_DECIMAL => {
                let (bigint, exp) = reader.read_bigdecimal()?.into_bigint_and_exponent();
                let coeff = bigint.iter_u64_digits().next().unwrap() as i64;
                let coeff = if bigint.sign() == Sign::Minus { -coeff } else { coeff };
                let value = coeff as f64 / (10f64).powi(exp as i32);
                Ok(IonValue::Float(value, annotations))
            }
            ION_TYPE_BOOL => Ok(IonValue::Boolean(reader.read_bool()?, annotations)),
            ION_TYPE_TIMESTAMP => {
                Ok(IonValue::Timestamp(reader.read_datetime()?.as_datetime().clone(), annotations))
            }
            _ => unreachable!()
        }
    }

    fn read_struct(reader: &mut IonCReaderHandle) -> IonResult<IonStruct> {
        reader.step_in()?;
        let mut fields = HashMap::new();
        loop {
            reader.next()?;
            match reader.get_type()? {
                ION_TYPE_NONE | ION_TYPE_EOF => {
                    reader.step_out()?;
                    return Ok(IonStruct::new(fields));
                }
                _ => {
                    let key = reader.get_field_name()?.as_str().to_string();
                    let value = IonReader::read_value(reader)?;
                    fields.insert(key, value);
                }
            }
        }
    }

    fn read_list(reader: &mut IonCReaderHandle) -> IonResult<IonList> {
        reader.step_in()?;
        let mut items = Vec::new();
        loop {
            reader.next()?;
            match reader.get_type()? {
                ION_TYPE_NONE | ION_TYPE_EOF => {
                    reader.step_out()?;
                    return Ok(IonList { items });
                }
                _ => {
                    let item = IonReader::read_value(reader)?;
                    items.push(item);
                }
            }
        }
    }
}