use mlua::{FromLua, Function, Lua, Table, Value};
use tracing::error;

use super::{Command, HttpRequest};
use crate::Result;

#[derive(Debug)]
pub struct SearchCommand {
    page: Function,
    parse: Function,
}

#[derive(Debug)]
pub struct SearchItem {
    pub id: String,
    pub title: String,
    pub author: String,
    pub cover: String,
    pub last_update: String,
    pub status: String,
    pub intro: String,
}

impl FromLua for SearchItem {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table: Table = lua.unpack(value)?;
        let id = table.get("id")?;
        let title = table.get("title")?;
        let author = table.get("author")?;
        let cover = table.get("cover")?;
        let last_update = table.get("last_update")?;
        let status = table.get("status")?;
        let intro = table.get("intro")?;
        Ok(SearchItem {
            id,
            title,
            author,
            cover,
            last_update,
            status,
            intro,
        })
    }
}

pub struct SearchItemIter {
    parse_fn: Function,
}

impl Iterator for SearchItemIter {
    type Item = SearchItem;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse_fn.call(()) {
            Ok(item) => item,
            Err(e) => {
                error!(error = %e, "parse a search item failed");
                None
            }
        }
    }
}

impl FromLua for SearchCommand {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table: Table = lua.unpack(value)?;
        let page = table.get("page")?;
        let parse = table.get("parse")?;
        Ok(SearchCommand { page, parse })
    }
}

impl Command for SearchCommand {
    type PagePath = Option<HttpRequest>;
    type Page = String;
    type PagePathParams = (u64, Option<Self::Page>);
    type PageContent = SearchItemIter;

    fn page(&self, id: &str, params: Self::PagePathParams) -> Result<Self::PagePath> {
        let page: Self::PagePath = self.page.call((id, params.0, params.1))?;
        Ok(page)
    }

    fn parse(&self, content: Self::Page) -> Result<Self::PageContent> {
        let content: Function = self.parse.call(content)?;
        Ok(SearchItemIter { parse_fn: content })
    }
}
