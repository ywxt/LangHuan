use crate::{http::HttpRequest, Result};
use mlua::{FromLua, IntoLua, Table};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

mod book_info;
mod chapter;
mod info_parser;
mod search;
mod session;
mod toc;

pub use book_info::*;
pub use chapter::*;
pub use search::*;
pub use session::*;
pub use toc::*;



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

impl IntoLua for HttpRequest {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let table = lua.create_table()?;
        table.set("url", self.url)?;
        table.set("method", self.method)?;
        table.set("headers", self.headers)?;
        table.set("body", self.body)?;
        table.into_lua(lua)
    }
}

#[derive(Debug)]
pub struct Schema {
    pub schema_info: SchemaInfo,
    pub book_search: SearchCommand,
    pub book_info: BookInfoCommand,
    pub book_chapter: ChapterCommand,
    pub book_toc: TocCommand,
    pub session: Option<SessionCommand>,
}

impl Schema {
    pub fn load(script: &str, table: Table) -> Result<Self> {
        let schema_info = SchemaInfo::from_str(script)?;
        let book_search = table.get("search")?;
        let book_info = table.get("book_info")?;
        let book_chapter = table.get("chapter")?;
        let book_toc = table.get("toc")?;
        let session = table.get("session")?;
        Ok(Schema {
            schema_info,
            book_search,
            book_info,
            book_chapter,
            book_toc,
            session,
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
    pub legal_domains: HashSet<String>,
}

impl FromStr for SchemaInfo {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut id = None;
        let mut name = None;
        let mut author = None;
        let mut description = None;
        let mut fany_version = None;
        let mut legal_domains = HashSet::new();
        for line in info_parser::parse_script(s) {
            let line = line?;
            match line.name {
                "id" => id = Some(line.value),
                "name" => name = Some(line.value),
                "author" => author = Some(line.value),
                "description" => description = Some(line.value),
                "fany_version" => fany_version = Some(line.value),
                "legal_domains" => {
                    legal_domains.insert(line.value.to_string());
                }
                _ => {
                    return Err(crate::Error::ParseError(format!(
                        "unknown field in the script: {}",
                        line.name
                    )));
                }
            }
        }
        Ok(SchemaInfo {
            id: id
                .map(|id| id.to_owned())
                .ok_or_else(|| crate::Error::ParseError("missing field: id".to_string()))?,
            name: name
                .map(|name| name.to_owned())
                .ok_or_else(|| crate::Error::ParseError("missing field: name".to_string()))?,
            author: author
                .map(|author| author.to_owned())
                .ok_or_else(|| crate::Error::ParseError("missing field: author".to_string()))?,
            description: description
                .map(|description| description.to_owned())
                .ok_or_else(|| {
                    crate::Error::ParseError("missing field: description".to_string())
                })?,
            fany_version: fany_version
                .map(|fany_version| fany_version.to_owned())
                .ok_or_else(|| {
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

#[cfg(test)]
mod tests {

    /// a macro to create a hashset
    macro_rules! hashset {
        ( $( $x:expr ),* ) => {
            {
                let mut set = ::std::collections::HashSet::new();
                $(
                    set.insert($x);
                )*
                set
            }
        };
    }
    use super::*;

    #[test]
    fn test_schema_info() {
        let script = r#"--@id: test
--@name: test_schema
--@author: test_author
--@description: test
--@fany_version: 1.0
--@legal_domains: test.com
--@legal_domains: test2.com

"#;
        let schema_info = SchemaInfo::from_str(script).unwrap();
        assert_eq!(schema_info.id, "test");
        assert_eq!(schema_info.name, "test_schema");
        assert_eq!(schema_info.author, "test_author");
        assert_eq!(schema_info.description, "test");
        assert_eq!(schema_info.fany_version, "1.0");
        assert_eq!(
            schema_info.legal_domains,
            hashset!["test.com".to_string(), "test2.com".to_string()]
        );
    }

    #[test]
    fn test_schema() {
        let script = r#"--@id: test
--@name: test_schema
--@author: test_author
--@description: test
--@fany_version: 1.0
--@legal_domains: test.com
--@legal_domains: test2.com

local function search()
end
local function book_info()
end
local function chapter()
end
local function toc()
end
local function session()
end
return {
    search = {page = search, parse = search},
    book_info = {page = book_info, parse = book_info},
    chapter = {page = chapter, parse = chapter},
    toc = {page = toc, parse = toc},
    session = {page = session, parse = session, wrap = session},
}
"#;
        let lua = mlua::Lua::new();
        let table = lua.load(script).eval::<Table>().unwrap();
        let schema = Schema::load(script, table).unwrap();
        assert_eq!(schema.schema_info.id, "test");
        assert_eq!(schema.schema_info.name, "test_schema");
        assert_eq!(schema.schema_info.author, "test_author");
        assert_eq!(schema.schema_info.description, "test");
        assert_eq!(schema.schema_info.fany_version, "1.0");
        assert_eq!(
            schema.schema_info.legal_domains,
            hashset!["test.com".to_string(), "test2.com".to_string()]
        );
    }
}
