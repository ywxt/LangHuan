use tracing::instrument;

use crate::{
    package::{self, Package},
    schema::Schema,
};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

static RUNTIME_PACKAGES: LazyLock<HashMap<&'static str, Box<dyn Package + Send + Sync>>> =
    LazyLock::new(|| {
        let mut packages = HashMap::new();
        #[cfg(feature = "pkg-json")]
        packages.insert(
            "json",
            Box::new(package::json::JsonParserPackage) as Box<dyn Package + Send + Sync>,
        );
        #[cfg(feature = "pkg-url-encoding")]
        packages.insert("url", Box::new(package::url::UrlPackage));
        packages
    });

#[derive(Debug, Clone)]
pub struct Runtime {
    lua: Arc<mlua::Lua>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
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
    #[instrument(skip(lua))]
    fn environment_require(name: &str, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let global = lua.globals();
        let package: mlua::Table = global.get("package")?;
        let loaded: mlua::Table = package.get("loaded")?;
        if let Some(value) = loaded.get(name)? {
            return Ok(value);
        }
        if !name.starts_with('@') {
            return Err(mlua::Error::RuntimeError(format!("invalid module name: {}, you can only import pre-defined modules that start with @", name)));
        }
        let package_name = &name[1..];
        if let Some(module) = Self::get_predefined_package(package_name) {
            let required = module.create_instance(lua)?;
            loaded.set(name, required.clone())?;
            return Ok(required);
        }
        Err(mlua::Error::RuntimeError(format!(
            "module not found: {}",
            name,
        )))
    }

    fn get_predefined_package(name: &str) -> Option<&'static (dyn Package + Send + Sync)> {
        RUNTIME_PACKAGES.get(name).map(|module| &**module)
    }
}

#[cfg(test)]
mod tests {
    use crate::hashset;

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
--@lh-version: 1.0
--@legal-domains: test.com


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
        assert_eq!(schema.schema_info.id, "test");
        assert_eq!(schema.schema_info.name, "test_schema");
        assert_eq!(schema.schema_info.author, "test_author");
        assert_eq!(schema.schema_info.description, "test");
        assert_eq!(schema.schema_info.lh_version, "1.0");
        assert_eq!(
            schema.schema_info.legal_domains,
            hashset!["test.com".to_string()]
        );
    }

    #[test]
    #[cfg(feature = "pkg-json")]
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
            assert(json == json1)
        "#,
            )
            .set_environment(env)
            .exec()
            .unwrap();
        let result = runtime.lua.load(r#"require('json')"#).exec();
        assert!(result.is_err());
    }
}
