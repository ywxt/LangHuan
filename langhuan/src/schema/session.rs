use mlua::{FromLua, Function};

use super::{Command, HttpRequest};

use crate::Result;

pub type Session = mlua::Value;

#[derive(Debug)]
pub struct SessionCommand {
    page: Function,
    parse: Function,
    wrap: Function,
}

impl SessionCommand {
    pub fn wrap(
        &self,
        page_path: <Self as Command>::Request,
        session: <Self as Command>::PageContent,
    ) -> Result<<Self as Command>::Request> {
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
    type Request = HttpRequest;

    type Page = String;
    type RequestParams = ();

    type PageContent = Session;

    fn parse(&self, content: Self::Page) -> Result<Self::PageContent> {
        Ok(self.parse.call(content)?)
    }

    fn page(&self, _: &str, _: Self::RequestParams) -> Result<Self::Request> {
        Ok(self.page.call(())?)
    }
}
