//! JSON conversion between Rust values and their representation in the view.
//!
//! Generated code implements [`ToJson`] for models and [`FromJson`] for models
//! and events; the view sends and receives the same shapes the JS host uses.

use std::fmt;

use serde_json::{Map, Number, Value};

/// Converts a value into the JSON the view understands.
pub trait ToJson {
    fn to_json(&self) -> Value;
}

/// Decodes a value sent by the view.
pub trait FromJson: Sized {
    /// # Errors
    ///
    /// Fails when `value` does not have the expected shape.
    fn from_json(value: &Value) -> Result<Self, DecodeError>;
}

/// A value sent by the view does not match the declared type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError {
    path: Vec<String>,
    expected: &'static str,
}

impl DecodeError {
    fn expected(expected: &'static str) -> Self {
        Self {
            path: Vec::new(),
            expected,
        }
    }

    /// Prepends a field or type name to the error's location.
    fn within(mut self, segment: String) -> Self {
        self.path.insert(0, segment);
        self
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, segment) in self.path.iter().enumerate() {
            if i > 0 && !segment.starts_with('[') {
                f.write_str(".")?;
            }
            f.write_str(segment)?;
        }
        if !self.path.is_empty() {
            f.write_str(": ")?;
        }
        write!(f, "expected {}", self.expected)
    }
}

impl std::error::Error for DecodeError {}

impl ToJson for () {
    fn to_json(&self) -> Value {
        Value::Null
    }
}

impl FromJson for () {
    fn from_json(_: &Value) -> Result<Self, DecodeError> {
        Ok(())
    }
}

impl ToJson for i64 {
    fn to_json(&self) -> Value {
        Value::from(*self)
    }
}

/// Upper bound (exclusive) of `f64` values convertible to `i64`: 2^63.
const I64_LIMIT: f64 = 9_223_372_036_854_775_808.0;

impl FromJson for i64 {
    fn from_json(value: &Value) -> Result<Self, DecodeError> {
        if let Some(int) = value.as_i64() {
            return Ok(int);
        }
        // JS numbers such as `1e21` or results of float arithmetic arrive as
        // floats; accept them when they hold an integral value in range.
        match value.as_f64() {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "the value is integral and within the i64 range"
            )]
            Some(float) if float.fract() == 0.0 && (-I64_LIMIT..I64_LIMIT).contains(&float) => {
                Ok(float as i64)
            }
            _ => Err(DecodeError::expected("an integer")),
        }
    }
}

impl ToJson for f64 {
    /// Non-finite values have no JSON representation and become `null`.
    fn to_json(&self) -> Value {
        Number::from_f64(*self).map_or(Value::Null, Value::Number)
    }
}

impl FromJson for f64 {
    fn from_json(value: &Value) -> Result<Self, DecodeError> {
        value.as_f64().ok_or(DecodeError::expected("a number"))
    }
}

impl ToJson for bool {
    fn to_json(&self) -> Value {
        Value::Bool(*self)
    }
}

impl FromJson for bool {
    fn from_json(value: &Value) -> Result<Self, DecodeError> {
        value.as_bool().ok_or(DecodeError::expected("a boolean"))
    }
}

impl ToJson for String {
    fn to_json(&self) -> Value {
        Value::String(self.clone())
    }
}

impl FromJson for String {
    fn from_json(value: &Value) -> Result<Self, DecodeError> {
        value
            .as_str()
            .map(str::to_owned)
            .ok_or(DecodeError::expected("a string"))
    }
}

impl<T: ToJson> ToJson for Vec<T> {
    fn to_json(&self) -> Value {
        Value::Array(self.iter().map(ToJson::to_json).collect())
    }
}

impl<T: FromJson> FromJson for Vec<T> {
    fn from_json(value: &Value) -> Result<Self, DecodeError> {
        let items = value.as_array().ok_or(DecodeError::expected("an array"))?;
        items
            .iter()
            .enumerate()
            .map(|(i, item)| T::from_json(item).map_err(|e| e.within(format!("[{i}]"))))
            .collect()
    }
}

impl<T: ToJson> ToJson for Option<T> {
    fn to_json(&self) -> Value {
        self.as_ref().map_or(Value::Null, ToJson::to_json)
    }
}

impl<T: FromJson> FromJson for Option<T> {
    fn from_json(value: &Value) -> Result<Self, DecodeError> {
        match value {
            Value::Null => Ok(None),
            value => T::from_json(value).map(Some),
        }
    }
}

/// Builds a JSON object from `(field, value)` pairs.
#[must_use]
pub fn object<const N: usize>(entries: [(&str, Value); N]) -> Value {
    let map: Map<String, Value> = entries
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect();
    Value::Object(map)
}

/// Fields of a JSON object being decoded into a model or event named `name`.
#[derive(Debug)]
pub struct Fields<'v> {
    map: &'v Map<String, Value>,
    name: &'static str,
}

impl<'v> Fields<'v> {
    /// # Errors
    ///
    /// Fails when `value` is not an object.
    pub fn new(value: &'v Value, name: &'static str) -> Result<Self, DecodeError> {
        match value {
            Value::Object(map) => Ok(Self { map, name }),
            _ => Err(DecodeError::expected("an object").within(name.to_owned())),
        }
    }

    /// Decodes `field`; a missing field reads as `null`.
    ///
    /// # Errors
    ///
    /// Fails when the field does not decode as `T`.
    pub fn get<T: FromJson>(&self, field: &str) -> Result<T, DecodeError> {
        T::from_json(self.map.get(field).unwrap_or(&Value::Null))
            .map_err(|e| e.within(field.to_owned()).within(self.name.to_owned()))
    }
}

/// Reached only if generated code records a change path its model lacks.
///
/// # Panics
///
/// Always.
#[cold]
pub fn unknown_path(model: &str, path: &[&str]) -> ! {
    panic!("weft: model `{model}` has no field path {path:?}")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn integers_accept_integral_floats_only() {
        assert_eq!(i64::from_json(&json!(3)), Ok(3));
        assert_eq!(i64::from_json(&json!(3.0)), Ok(3));
        assert!(i64::from_json(&json!(3.5)).is_err());
        assert!(i64::from_json(&json!(1e300)).is_err());
    }

    #[test]
    fn decode_errors_name_the_field() {
        let value = json!({ "items": [1, "x"] });
        let fields = Fields::new(&value, "Event").expect("an object");
        let error = fields
            .get::<Vec<i64>>("items")
            .expect_err("a string is not an integer");
        assert_eq!(error.to_string(), "Event.items[1]: expected an integer");
        assert_eq!(fields.get::<Option<i64>>("missing"), Ok(None));
    }
}
