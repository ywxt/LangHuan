use crate::Result;
use mlua::{FromLua, Table};
use std::{collections::HashMap, str::FromStr};

mod book_info;
mod chapter;
mod info_parser;
mod search;
mod toc;

pub use book_info::*;
pub use chapter::*;
pub use info_parser::*;
pub use search::*;
pub use toc::*;

#[derive(Debug)]
pub struct HttpRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

impl FromLua for HttpRequest {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        if let mlua::Value::String(url) = value {
            Ok(HttpRequest {
                url: url.to_str()?.to_string(),
                method: "GET".to_string(),
                headers: HashMap::new(),
                body: None,
            })
        } else {
            let table: mlua::Table = lua.unpack(value)?;
            let url: String = table.get("url")?;
            let method = table.get("method")?;
            let headers = table.get("headers")?;
            let body = table.get("body")?;
            Ok(HttpRequest {
                url,
                method,
                headers,
                body,
            })
        }
    }
}

#[derive(Debug)]
pub struct Schema {
    pub schema_info: SchemaInfo,
    pub book_search: SearchCommand,
    pub book_info: BookInfoCommand,
    pub book_chapter: ChapterCommand,
    pub book_toc: TocCommand,
}

impl Schema {
    pub fn load(script: &str, table: Table) -> Result<Self> {
        let schema_info = SchemaInfo::from_str(script)?;
        let book_search = table.get("search")?;
        let book_info = table.get("book_info")?;
        let book_chapter = table.get("chapter")?;
        let book_toc = table.get("toc")?;
        Ok(Schema {
            schema_info,
            book_search,
            book_info,
            book_chapter,
            book_toc,
        })
    }
}

#[derive(Debug)]
pub struct SchemaInfo {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub fany_version: String,
    pub legal_domains: Vec<String>,
}

impl FromStr for SchemaInfo {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut id = None;
        let mut name = None;
        let mut author = None;
        let mut description = None;
        let mut fany_version = None;
        let mut legal_domains = Vec::new();
        for line in parse_script(s)? {
            match line.name.as_str() {
                "id" => id = Some(line.value),
                "name" => name = Some(line.value),
                "author" => author = Some(line.value),
                "description" => description = Some(line.value),
                "fany_version" => fany_version = Some(line.value),
                "legal_domains" => legal_domains.push(line.value.to_string()),
                _ => Err(crate::Error::ParseError(format!(
                    "unknown field in the script: {}",
                    line.name
                )))?,
            }
        }
        Ok(SchemaInfo {
            id: id.ok_or_else(|| crate::Error::ParseError("missing field: id".to_string()))?,
            name: name
                .ok_or_else(|| crate::Error::ParseError("missing field: name".to_string()))?,
            author: author
                .ok_or_else(|| crate::Error::ParseError("missing field: author".to_string()))?,
            description: description.ok_or_else(|| {
                crate::Error::ParseError("missing field: description".to_string())
            })?,
            fany_version: fany_version.ok_or_else(|| {
                crate::Error::ParseError("missing field: fany_version".to_string())
            })?,
            legal_domains,
        })
    }
}

pub trait Command: FromLua {
    type PagePath;
    type Page;
    type PagePathParams;
    type PageContent;
    fn page(&self, id: &str, params: Self::PagePathParams) -> Result<Self::PagePath>;
    fn parse(&self, content: Self::Page) -> Result<Self::PageContent>;
}
