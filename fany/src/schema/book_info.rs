use mlua::{FromLua, Function, LuaSerdeExt};
use serde::Deserialize;

use super::{Command, HttpRequest};

use crate::{http::HttpClient, Result};

#[derive(Debug)]
pub struct BookInfoCommand {
    page: Function,
    parse: Function,
}

impl BookInfoCommand {
    pub async fn get_info(&self, id: &str, http: &HttpClient) -> Result<BookInfo> {
        let request: HttpRequest = self.page(id, ())?;
        let content = http.request(request).await?;
        let content = self.parse(content)?;
        Ok(content)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashset;
    use crate::http::HttpClient;
    use mlua::prelude::*;

    #[tokio::test]
    async fn test_book_info() {
        let lua = Lua::new();
        let client = reqwest::Client::new();
        let http = HttpClient::new(client, hashset!["www.example.com".to_string()]);
        let code = r#"
            {
                page = function(id)
                    return "http://www.example.com/" .. id
                end,
                parse = function(content)
                    return {
                        title = "title",
                        author = "author",
                        cover = "cover",
                        last_update = "last_update",
                        status = "status",
                        intro = "intro",
                    }
                end,
            }
        "#;
        let command: BookInfoCommand = lua.load(code).eval().unwrap();
        let info = command.get_info("123", &http).await.unwrap();
        assert_eq!(info.title, "title");
        assert_eq!(info.author, "author");
        assert_eq!(info.cover, "cover");
        assert_eq!(info.last_update, "last_update");
        assert_eq!(info.status, "status");
        assert_eq!(info.intro, "intro");
    }
}
