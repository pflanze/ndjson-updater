use anyhow::{bail, Result, anyhow};
use jzon::{JsonValue, object::Object};

pub trait EasyJsonValue {
    fn string(&self) -> Result<String>;
    fn str(&self) -> Result<&str>;
    fn object(&self) -> Result<&Object>;
    fn object_mut(&mut self) -> Result<&mut Object>;
}

impl EasyJsonValue for JsonValue {
    fn string(&self) -> Result<String> {
        Ok(self.str()?.into())
    }

    fn str(&self) -> Result<&str> {
        match self {
            JsonValue::Null => bail!("got null where string expected"),
            JsonValue::Short(s) => Ok(s),
            JsonValue::String(s) => Ok(s),
            JsonValue::Number(v) => bail!("got number where string expected: {v}"),
            JsonValue::Boolean(v) => bail!("got boolean where string expected: {v}"),
            JsonValue::Object(_) => bail!("got object where string expected"),
            JsonValue::Array(_) => bail!("got array where string expected"),
        }
    }

    fn object(&self) -> Result<&Object> {
        match self {
            JsonValue::Null => bail!("got null where object expected"),
            JsonValue::Short(s) => bail!("got string where object expected: {s:?}"),
            JsonValue::String(s) => bail!("got string where object expected: {s:?}"),
            JsonValue::Number(v) => bail!("got number where object expected: {v}"),
            JsonValue::Boolean(v) => bail!("got boolean where object expected: {v}"),
            JsonValue::Object(v) => Ok(v),
            JsonValue::Array(_) => bail!("got array where object expected"),
        }
    }

    fn object_mut(&mut self) -> Result<&mut Object> {
        match self {
            JsonValue::Null => bail!("got null where object expected"),
            JsonValue::Short(s) => bail!("got string where object expected: {s:?}"),
            JsonValue::String(s) => bail!("got string where object expected: {s:?}"),
            JsonValue::Number(v) => bail!("got number where object expected: {v}"),
            JsonValue::Boolean(v) => bail!("got boolean where object expected: {v}"),
            JsonValue::Object(v) => Ok(v),
            JsonValue::Array(_) => bail!("got array where object expected"),
        }
    }
}


pub trait EasyObject {
    /// just str keys for now
    fn xget(&self, key: &str) -> Result<&JsonValue>;
    fn xget_mut(&mut self, key: &str) -> Result<&mut JsonValue>;
}

impl EasyObject for Object {
    fn xget(&self, key: &str) -> Result<&JsonValue> {
        self.get(key).ok_or_else(
            || anyhow!("missing key {key:?}"))
    }
    fn xget_mut(&mut self, key: &str) -> Result<&mut JsonValue> {
        self.get_mut(key).ok_or_else(
            || anyhow!("missing key {key:?}"))
    }
}

