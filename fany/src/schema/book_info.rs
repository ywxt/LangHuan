use mlua::{FromLua, Function};

use super::{Command, HttpRequest};

use crate::Result;

#[derive(Debug)]
pub struct BookInfoCommand {
    page: Function,
    parse: Function,
}

#[derive(Debug)]
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
        let table: mlua::Table = lua.unpack(value)?;
        let title = table.get("title")?;
        let author = table.get("author")?;
        let cover = table.get("cover")?;
        let last_update = table.get("last_update")?;
        let status = table.get("status")?;
        let intro = table.get("intro")?;
        Ok(BookInfo {
            title,
            author,
            cover,
            last_update,
            status,
            intro,
        })
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
