use std::ops::Deref;

use mlua::{FromLua, UserData};

#[cfg(feature = "pkg-json")]
pub mod json;
#[cfg(feature = "pkg-url-encoding")]
pub mod url;

#[derive(Debug, Clone)]
struct Bytes(bytes::Bytes);

impl UserData for Bytes {}

impl Deref for Bytes {
    type Target = bytes::Bytes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromLua for Bytes {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        if let mlua::Value::UserData(ud) = value {
            Ok(ud.borrow::<Bytes>()?.clone())
        } else {
            Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Bytes".to_string(),
                message: Some("value is not a Bytes".to_string()),
            })
        }
    }
}

pub trait Package {
    fn create_instance(&self, lua: &mlua::Lua) -> mlua::Result<mlua::Value>;
}
