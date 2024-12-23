use mlua::{ExternalError, IntoLua, LuaSerdeExt, UserData};

use super::{Bytes, Package};

#[derive(Debug, Clone, Default)]
pub struct JsonParserPackage;

impl Package for JsonParserPackage {
    fn create_instance(&self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        Self.into_lua(lua)
    }
}

impl UserData for JsonParserPackage {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("decode_utf8", |lua, json: Bytes| {
            let value: serde_json::Value =
                serde_json::from_slice(&json).map_err(|e| e.into_lua_err())?;
            let options = mlua::SerializeOptions::new()
                .serialize_none_to_null(false)
                .serialize_unit_to_null(false)
                .set_array_metatable(false)
                .detect_serde_json_arbitrary_precision(true);
            lua.to_value_with(&value, options)
        });
        methods.add_function("decode", |lua, json: String| {
            let value: serde_json::Value =
                serde_json::from_str(&json).map_err(|e| e.into_lua_err())?;
            let options = mlua::SerializeOptions::new()
                .serialize_none_to_null(false)
                .serialize_unit_to_null(false)
                .set_array_metatable(false)
                .detect_serde_json_arbitrary_precision(true);
            lua.to_value_with(&value, options)
        });
        methods.add_function("encode", |_, value: mlua::Value| {
            serde_json::to_string(&value).map_err(|e| e.into_lua_err())
        });
        methods.add_function("stringify", |_, value: mlua::Value| {
            serde_json::to_string_pretty(&value).map_err(|e| e.into_lua_err())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::prelude::*;

    #[test]
    fn test_decode() {
        let lua = Lua::new();
        let module = JsonParserPackage.into_lua(&lua).unwrap();
        lua.globals().set("json", module).unwrap();
        let _: () = lua
            .load(
                r#"
                local value = json.decode('{"a": 1, "b": [1, 2, 3], "c": {"d": 4, "f": null}}')
                assert(value['a'] == 1)
                assert(#value['b'] == 3)
                assert(value['b'][1] == 1)
                assert(value['b'][2] == 2)
                assert(value['b'][3] == 3)
                assert(value['c']['d'] == 4)
                assert(value['c']['f'] == nil)
            "#,
            )
            .eval()
            .unwrap();
    }

    #[test]
    fn test_encode() {
        let lua = Lua::new();
        let module = JsonParserPackage.into_lua(&lua).unwrap();
        lua.globals().set("json", module).unwrap();
        let _: () = lua
            .load(
                r#"
                local value = {a = 1, b = {1, 2, 3}, c = {d = 4, f = nil}}
                local json_str = json.encode(value)
                assert(string.find(json_str, '"a":1', 1, true))
                assert(string.find(json_str, '"b":[1,2,3]', 1, true))
                assert(string.find(json_str, '"c":{"d":4}', 1, true))
            "#,
            )
            .eval()
            .unwrap();
    }
}
