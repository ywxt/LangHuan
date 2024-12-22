use mlua::{FromLua, Function, Lua, Table, Value};
use tracing::error;

use super::{Command, HttpRequest};
use crate::Result;

#[derive(Debug)]
pub struct ChapterCommand {
    page: Function,
    parse: Function,
}

#[derive(Debug)]
pub enum Paragraph {
    Text(String),
    Image(String),
}

impl FromLua for Paragraph {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table: Table = lua.unpack(value)?;
        let r#type: String = table.get("type")?;
        let content: String = table.get("content")?;
        match r#type.as_str() {
            "text" => Ok(Paragraph::Text(content)),
            "image" => Ok(Paragraph::Image(content)),
            _ => Err(mlua::Error::external("unknown paragraph type")),
        }
    }
}

pub struct ParagraphIter {
    parse_fn: Function,
}

impl Iterator for ParagraphIter {
    type Item = Result<Paragraph>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_fn
            .call(())
            .map_err(|e| {
                error!("parse paragraph failed: {}", e);
                e.into()
            })
            .transpose()
    }
}

impl FromLua for ChapterCommand {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table: Table = lua.unpack(value)?;
        let page = table.get("page")?;
        let parse = table.get("parse")?;
        Ok(ChapterCommand { page, parse })
    }
}

impl Command for ChapterCommand {
    type Request = Option<HttpRequest>;
    type Page = String;
    type RequestParams = (u64, Option<Self::Page>);
    type PageContent = ParagraphIter;

    fn page(&self, id: &str, params: Self::RequestParams) -> Result<Self::Request> {
        let page: Self::Request = self.page.call((id, params.0, params.1))?;
        Ok(page)
    }

    fn parse(&self, content: Self::Page) -> Result<Self::PageContent> {
        let content: Function = self.parse.call(content)?;
        Ok(ParagraphIter { parse_fn: content })
    }
}
