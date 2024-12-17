use mlua::{FromLua, Function, LuaSerdeExt};
use serde::Deserialize;

use super::{Command, HttpRequest};

use crate::Result;

#[derive(Debug)]
pub struct BookInfoCommand {
    page: Function,
    parse: Function,
}

#[derive(Debug, Deserialize)]
pub struct BookInfo {
    pub title: String,
    pub author: String,
    pub cover: String,
    pub last_update: String,
    pub status: String,
    pub intro: String,
}

impl FromLua for BookInfo {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}

impl FromLua for BookInfoCommand {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        let table: mlua::Table = lua.unpack(value)?;
        let page = table.get("page")?;
        let parse = table.get("parse")?;
        Ok(BookInfoCommand { page, parse })
    }
}

impl Command for BookInfoCommand {
    type PagePath = HttpRequest;

    type Page = String;
    type PagePathParams = ();

    type PageContent = BookInfo;

    fn parse(&self, content: Self::Page) -> Result<Self::PageContent> {
        Ok(self.parse.call(content)?)
    }

    fn page(&self, id: &str, _: Self::PagePathParams) -> Result<Self::PagePath> {
        Ok(self.page.call(id)?)
    }
}
