use mlua::{FromLua, Function};

use super::{Command, HttpRequest};

use crate::Result;

#[derive(Debug)]
pub struct SessionCommand {
    page: Function,
    parse: Function,
    wrap: Function,
}

impl SessionCommand {
    pub fn wrap(
        &self,
        page_path: <Self as Command>::PagePath,
        session: <Self as Command>::PageContent,
    ) -> Result<<Self as Command>::PagePath> {
        Ok(self.wrap.call((page_path, session))?)
    }
}

impl FromLua for SessionCommand {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        let table: mlua::Table = lua.unpack(value)?;
        let page = table.get("page")?;
        let parse = table.get("parse")?;
        let wrap = table.get("wrap")?;
        Ok(SessionCommand { page, parse, wrap })
    }
}

impl Command for SessionCommand {
    type PagePath = HttpRequest;

    type Page = String;
    type PagePathParams = ();

    type PageContent = mlua::Value;

    fn parse(&self, content: Self::Page) -> Result<Self::PageContent> {
        Ok(self.parse.call(content)?)
    }

    fn page(&self, _: &str, _: Self::PagePathParams) -> Result<Self::PagePath> {
        Ok(self.page.call(())?)
    }
}
