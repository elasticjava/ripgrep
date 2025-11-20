use std::borrow::Cow;
use std::fmt;

/// A value that can be stored in metadata.
///
/// Supports common types used in document metadata:
/// strings, integers, floating-point numbers, and booleans.
#[derive(Debug, Clone, PartialEq)]
pub enum MetaValue {
    /// A string value (static or owned)
    Str(Cow<'static, str>),
    /// A 64-bit signed integer
    Int(i64),
    /// A 64-bit floating-point number
    Float(f64),
    /// A boolean value
    Bool(bool),
}

impl fmt::Display for MetaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetaValue::Str(s) => write!(f, "{}", s),
            MetaValue::Int(i) => write!(f, "{}", i),
            MetaValue::Float(fl) => write!(f, "{}", fl),
            MetaValue::Bool(b) => write!(f, "{}", b),
        }
    }
}

// Convenience From implementations
impl From<&'static str> for MetaValue {
    fn from(s: &'static str) -> Self {
        MetaValue::Str(Cow::Borrowed(s))
    }
}

impl From<String> for MetaValue {
    fn from(s: String) -> Self {
        MetaValue::Str(Cow::Owned(s))
    }
}

impl From<i64> for MetaValue {
    fn from(i: i64) -> Self {
        MetaValue::Int(i)
    }
}

impl From<f64> for MetaValue {
    fn from(f: f64) -> Self {
        MetaValue::Float(f)
    }
}

impl From<bool> for MetaValue {
    fn from(b: bool) -> Self {
        MetaValue::Bool(b)
    }
}
