use crate::{
    module::{self, Module},
    schema::Schema,
};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

static RUNTIME_MODULES: LazyLock<HashMap<&'static str, Box<dyn Module + Send + Sync>>> =
    LazyLock::new(|| {
        let mut modules = HashMap::new();
        modules.insert(
            "json",
            Box::new(module::json::JsonParserModule) as Box<dyn Module + Send + Sync>,
        );
        modules
    });

#[derive(Debug, Clone)]
pub struct Runtime {
    lua: Arc<mlua::Lua>,
}

impl Runtime {
    pub fn new() -> Self {
        let lua = mlua::Lua::new();
        lua.sandbox(true).expect("enable sandbox failed");
        Self { lua: Arc::new(lua) }
    }

    pub fn load(&self, code: &str, name: &str) -> Result<Schema, crate::Error> {
        let chunk = self
            .lua
            .load(code)
            .set_name(format!("={}", name))
            .set_environment(self.create_environment()?);
        let result = chunk.eval()?;
        Schema::load(code, result)
    }

    fn create_environment(&self) -> mlua::Result<mlua::Table> {
        let env = self.lua.create_table()?;
        let globals = self.lua.globals();
        env.set_metatable(globals.metatable());
        let lua = self.lua.clone();
        env.raw_set(
            "require",
            self.lua
                .create_function(move |_, name: String| Self::environment_require(&name, &lua))?,
        )?;
        env.set_readonly(true);
        Ok(env)
    }

    fn environment_require(name: &str, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        if !name.starts_with('@') {
            return Err(mlua::Error::RuntimeError(format!("invalid module name: {}, you can only import pre-defined modules that start with @", name)));
        }
        let module_name = &name[1..];
        if let Some(module) = Self::get_predefined_module(module_name) {
            return module.create_instance(lua);
        }
        Err(mlua::Error::RuntimeError(format!(
            "module not found: {}",
            name,
        )))
    }

    fn get_predefined_module(name: &str) -> Option<&'static (dyn Module + Send + Sync)> {
        RUNTIME_MODULES.get(name).map(|module| &**module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime() {
        let runtime = Runtime::new();
        let schema = runtime
            .load(
                r#"--@id: test
--@name: test_schema
--@author: test_author
--@description: test
--@fany_version: 1.0
--@legal_domains: test.com


local function test() end
return {
    search = {page = test, parse = test},
    book_info = {page = test, parse = test},
    toc = {page = test, parse = test},
    chapter = {page = test, parse = test},
}
"#,
                "test",
            )
            .unwrap();
    }

    #[test]
    fn test_require() {
        let runtime = Runtime::new();
        let env = runtime.create_environment().unwrap();
        runtime
            .lua
            .load(
                r#"
            local json = require('@json')
            assert(json)
            assert(json.encode)
            assert(json.decode)
            assert(json.stringify)
            local json1 = require('@json')
            assert(json ~= json1)
        "#,
            )
            .set_environment(env)
            .exec()
            .unwrap();
        let result = runtime.lua.load(r#"require('json')"#).exec();
        assert!(result.is_err());
    }
}
