use mlua::{FromLua, Function, Lua, Table, Value};
use tracing::error;

use super::{Command, HttpRequest};
use crate::Result;

#[derive(Debug)]
pub struct TocCommand {
    page: Function,
    parse: Function,
}

#[derive(Debug)]
pub struct TocItem {
    pub title: String,
    pub id: String,
    pub tags: Vec<String>,
}

impl FromLua for TocItem {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table: Table = lua.unpack(value)?;
        let title: String = table.get("title")?;
        let id: String = table.get("id")?;
        let tags: Vec<String> = table.get("tags")?;
        Ok(TocItem { title, id, tags })
    }
}

pub struct TocItemIter {
    parse_fn: Function,
}

impl Iterator for TocItemIter {
    type Item = TocItem;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse_fn.call(()) {
            Ok(item) => item,
            Err(e) => {
                error!(error = %e, "parse a TOC item failed");
                None
            }
        }
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
    type PagePath = Option<HttpRequest>;
    type Page = String;
    type PagePathParams = (u64, Option<Self::Page>);
    type PageContent = TocItemIter;

    fn page(&self, id: &str, params: Self::PagePathParams) -> Result<Self::PagePath> {
        let page: Self::PagePath = self.page.call((id, params.0, params.1))?;
        Ok(page)
    }

    fn parse(&self, content: Self::Page) -> Result<Self::PageContent> {
        let content: Function = self.parse.call(content)?;
        Ok(TocItemIter { parse_fn: content })
    }
}
