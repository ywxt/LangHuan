use mlua::{FromLua, Function, Lua, Table, Value};
use tracing::error;

use super::{Command, HttpRequest};
use crate::{http::HttpClient, Result};

#[derive(Debug)]
pub struct SearchCommand {
    page: Function,
    parse: Function,
}

pub struct SearchItems<'a, 'b, 'c> {
    command: &'a SearchCommand,
    keyword: &'b str,
    page: u64,
    page_content: Option<String>,
    http: &'c HttpClient,
}

impl<'a, 'b, 'c> SearchItems<'a, 'b, 'c> {
    pub async fn next_page(&mut self) -> Result<Option<SearchItemIter>> {
        let request = self
            .command
            .page(self.keyword, (self.page, self.page_content.take()));
        match request {
            Err(e) => {
                error!("get search page failed: {}", e);
                Err(e)
            }
            Ok(None) => Ok(None),
            Ok(Some(request)) => {
                let response = self.http.request(&request).await?;
                let iter = self.command.parse(response.clone())?;
                self.page_content = Some(response);
                self.page += 1;
                Ok(Some(iter))
            }
        }
    }
}

impl SearchCommand {
    pub async fn search<'a, 'b, 'c>(
        &'a self,
        keyword: &'b str,
        http: &'c HttpClient,
    ) -> SearchItems<'a, 'b, 'c> {
        SearchItems {
            command: self,
            keyword,
            page: 1,
            page_content: None,
            http,
        }
    }
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
    type Item = Result<SearchItem>;

    fn next(&mut self) -> Option<Self::Item> {
        let result: mlua::Result<Option<SearchItem>> = self.parse_fn.call(());
        result
            .map_err(|e| {
                error!("parse search item failed: {}", e);
                e.into()
            })
            .transpose()
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
