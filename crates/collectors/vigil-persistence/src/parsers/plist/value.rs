use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum PlistValue {
    Dictionary(BTreeMap<String, PlistValue>),
    Array(Vec<PlistValue>),
    String(String),
    Integer(i128),
    Real(f64),
    Boolean(bool),
    Date(String),
    Data(usize),
    Uid(u64),
}

impl PlistValue {
    pub fn get(&self, key: &str) -> Option<&PlistValue> {
        match self {
            PlistValue::Dictionary(entries) => entries.get(key),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            PlistValue::String(text) => Some(text),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PlistValue::Boolean(said) => Some(*said),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i128> {
        match self {
            PlistValue::Integer(number) => Some(*number),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[PlistValue]> {
        match self {
            PlistValue::Array(values) => Some(values),
            _ => None,
        }
    }

    pub fn is_dictionary(&self) -> bool {
        matches!(self, PlistValue::Dictionary(_))
    }
}
