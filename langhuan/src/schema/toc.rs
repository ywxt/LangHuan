use mlua::{FromLua, Function, Lua, LuaSerdeExt, Table, Value};
use serde::Deserialize;
use tracing::error;

use super::{Command, HttpRequest};
use crate::Result;

#[derive(Debug)]
pub struct TocCommand {
    page: Function,
    parse: Function,
}

#[derive(Debug, Deserialize)]
pub struct TocItem {
    pub title: String,
    pub id: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl FromLua for TocItem {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}

pub struct TocItemIter {
    parse_fn: Function,
}

impl Iterator for TocItemIter {
    type Item = Result<TocItem>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_fn
            .call(())
            .map_err(|e| {
                error!("search item failed: {}", e);
                e.into()
            })
            .transpose()
    }
}

impl FromLua for TocCommand {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table: Table = lua.unpack(value)?;
        let page = table.get("page")?;
        let parse = table.get("parse")?;
        Ok(TocCommand { page, parse })
    }
}

impl Command for TocCommand {
    type Request = Option<HttpRequest>;
    type Page = String;
    type RequestParams = (u64, Option<Self::Page>);
    type PageContent = TocItemIter;

    fn page(&self, id: &str, params: Self::RequestParams) -> Result<Self::Request> {
        let page: Self::Request = self.page.call((id, params.0, params.1))?;
        Ok(page)
    }

    fn parse(&self, content: Self::Page) -> Result<Self::PageContent> {
        let content: Function = self.parse.call(content)?;
        Ok(TocItemIter { parse_fn: content })
    }
}
