use std::borrow::Cow;

use mlua::{IntoLua, UserData};

use super::Package;

#[derive(Debug, Default)]
pub struct UrlPackage;

impl Package for UrlPackage {
    fn create_instance(&self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        Self.into_lua(lua)
    }
}

impl UserData for UrlPackage {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("encode", |_, (text, encoding): (String, Option<String>)| {
            let encoding_label = encoding.as_deref().unwrap_or("utf-8");
            let encoding_label = encoding_rs::Encoding::for_label(encoding_label.as_bytes())
                .ok_or_else(|| {
                    mlua::Error::external(format!("invalid encoding:{}", encoding_label))
                })?;
            let (encoded, _, _) = encoding_label.encode(&text);
            Ok(
                percent_encoding::percent_encode(&encoded, percent_encoding::NON_ALPHANUMERIC)
                    .to_string(),
            )
        });
        methods.add_function("decode", |_, (text, encoding): (String, Option<String>)| {
            let text: Cow<'_, [u8]> = percent_encoding::percent_decode_str(&text).into();
            let encoding_label = encoding.as_deref().unwrap_or("utf-8");
            let encoding_label = encoding_rs::Encoding::for_label(encoding_label.as_bytes())
                .ok_or_else(|| {
                    mlua::Error::external(format!("invalid encoding:{}", encoding_label))
                })?;
            let (decoded, _, _) = encoding_label.decode(&text);
            Ok(decoded.into_owned())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let lua = mlua::Lua::new();
        let package = UrlPackage;
        let instance = package.create_instance(&lua).unwrap();
        lua.globals().set("url", instance).unwrap();
        let result: String = lua
            .load(
                r#"
                return url.encode("Hello 你好")
            "#,
            )
            .eval()
            .unwrap();
        assert_eq!(result, "Hello%20%E4%BD%A0%E5%A5%BD");
        let result: String = lua
            .load(
                r#"
                return url.encode("Hello 你好", "gbk")
            "#,
            )
            .eval()
            .unwrap();
        assert_eq!(result, "Hello%20%C4%E3%BA%C3");
    }

    #[test]
    fn test_decode() {
        let lua = mlua::Lua::new();
        let package = UrlPackage;
        let instance = package.create_instance(&lua).unwrap();
        lua.globals().set("url", instance).unwrap();
        let result: String = lua
            .load(
                r#"
                return url.decode("Hello%20%E4%BD%A0%E5%A5%BD")
            "#,
            )
            .eval()
            .unwrap();
        assert_eq!(result, "Hello 你好");
        let result: String = lua
            .load(
                r#"
                return url.decode("Hello%20%C4%E3%BA%C3", "gbk")
            "#,
            )
            .eval()
            .unwrap();
        assert_eq!(result, "Hello 你好");
    }
}
