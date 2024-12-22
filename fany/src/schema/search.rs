use mlua::{FromLua, Function, Lua, LuaSerdeExt, Table, Value};
use serde::Deserialize;
use tracing::error;

use super::{Command, HttpRequest};
use crate::Result;

#[derive(Debug)]
pub struct SearchCommand {
    page: Function,
    parse: Function,
}

#[derive(Debug, Deserialize)]
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
        lua.from_value(value)
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
    type Request = Option<HttpRequest>;
    type Page = String;
    type RequestParams = (u64, Option<Self::Page>);
    type PageContent = SearchItemIter;

    fn page(&self, id: &str, params: Self::RequestParams) -> Result<Self::Request> {
        let page: Self::Request = self.page.call((id, params.0, params.1))?;
        Ok(page)
    }

    fn parse(&self, content: Self::Page) -> Result<Self::PageContent> {
        let content: Function = self.parse.call(content)?;
        Ok(SearchItemIter { parse_fn: content })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashset;
    use crate::http::HttpClient;
    use crate::schema::PageItems;

    #[tokio::test]
    async fn test_search() {
        let lua = Lua::new();
        let allowed_domains = hashset!["www.example.com".to_string()];
        let http = HttpClient::new(reqwest::Client::new(), allowed_domains);
        let search = lua
            .load(
                r#"
                {
                    page = function(keyword, page, content)
                        if page == 1 then
                            return {
                                url = "https://www.example.com",
                                method = "GET",
                                headers = {},
                            }
                        end
                    end,
                    parse = function(content)
                        return function()
                            return {
                                id = "1",
                                title = "title",
                                author = "author",
                                cover = "cover",
                                last_update = "last_update",
                                status = "status",
                                intro = "intro",
                            }
                        end
                    end,
                }
            "#,
            )
            .eval::<SearchCommand>();
        let search = search.unwrap();
        let mut items = PageItems {
            command: &search,
            keyword: "keyword",
            page: 1,
            page_content: None,
            http: &http,
        };
        let item = items
            .next_page()
            .await
            .unwrap()
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        assert_eq!(item.id, "1");
        assert_eq!(item.title, "title");
        assert_eq!(item.author, "author");
        assert_eq!(item.cover, "cover");
        assert_eq!(item.last_update, "last_update");
        assert_eq!(item.status, "status");
        assert_eq!(item.intro, "intro");
    }
}
