use crate::{
    http::{HttpClient, HttpRequest},
    Result,
};
use mlua::{FromLua, IntoLua, LuaSerdeExt, Table};
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
                method: Default::default(),
                headers: HashMap::new(),
                body: None,
            })
        } else {
            lua.from_value(value)
        }
    }
}

impl IntoLua for HttpRequest {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let options = mlua::SerializeOptions::new()
            .serialize_none_to_null(true)
            .serialize_unit_to_null(true)
            .set_array_metatable(false);
        lua.to_value_with(&self, options)
    }
}

trait CommandRequest {
    fn wrap(self, map: impl FnOnce(HttpRequest) -> Result<HttpRequest>) -> Result<Self>
    where
        Self: Sized;
}

impl CommandRequest for HttpRequest {
    fn wrap(self, map: impl FnOnce(HttpRequest) -> Result<HttpRequest>) -> Result<Self>
    where
        Self: Sized,
    {
        map(self)
    }
}

impl CommandRequest for Option<HttpRequest> {
    fn wrap(self, map: impl FnOnce(HttpRequest) -> Result<HttpRequest>) -> Result<Self>
    where
        Self: Sized,
    {
        match self {
            Some(request) => map(request).map(Some),
            None => Ok(None),
        }
    }
}

#[derive(Debug)]
pub struct Schema {
    pub schema_info: SchemaInfo,
    book_search: SearchCommand,
    book_info: BookInfoCommand,
    book_chapter: ChapterCommand,
    book_toc: TocCommand,
    session: Option<SessionCommand>,
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

    pub fn search<'a, 'b, 'c>(
        &'a self,
        keyword: &'b str,
        http: &'c HttpClient,
        session: Option<Session>,
    ) -> SearchItems<'b, 'c, CommandWithSession<'a, 'a, SearchCommand>> {
        let command = CommandWithSession::new(&self.book_search, self.session.as_ref(), session);
        SearchItems::new(command, keyword, http)
    }

    pub async fn book_info<'a, 'b, 'c>(
        &'a self,
        id: &'b str,
        http: &'c HttpClient,
        session: Option<Session>,
    ) -> Result<BookInfo> {
        let command = CommandWithSession::new(&self.book_info, self.session.as_ref(), session);
        let path = command.page(id, ())?;
        let content = http.request(path).await?;
        command.parse(content)
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
                    return Err(crate::Error::ScriptParseError(format!(
                        "unknown field in the script: {}",
                        line.name
                    )));
                }
            }
        }
        Ok(SchemaInfo {
            id: id
                .map(|id| id.to_owned())
                .ok_or_else(|| crate::Error::ScriptParseError("missing field: id".to_string()))?,
            name: name
                .map(|name| name.to_owned())
                .ok_or_else(|| crate::Error::ScriptParseError("missing field: name".to_string()))?,
            author: author.map(|author| author.to_owned()).ok_or_else(|| {
                crate::Error::ScriptParseError("missing field: author".to_string())
            })?,
            description: description
                .map(|description| description.to_owned())
                .ok_or_else(|| {
                    crate::Error::ScriptParseError("missing field: description".to_string())
                })?,
            fany_version: fany_version
                .map(|fany_version| fany_version.to_owned())
                .ok_or_else(|| {
                    crate::Error::ScriptParseError("missing field: fany_version".to_string())
                })?,
            legal_domains,
        })
    }
}

pub trait Command {
    type PagePath;
    type Page;
    type PagePathParams;
    type PageContent;
    fn page(&self, id: &str, params: Self::PagePathParams) -> Result<Self::PagePath>;
    fn parse(&self, content: Self::Page) -> Result<Self::PageContent>;
}

impl<C> Command for &C
where
    C: Command,
{
    type Page = C::Page;
    type PageContent = C::PageContent;
    type PagePath = C::PagePath;
    type PagePathParams = C::PagePathParams;

    fn page(&self, id: &str, params: C::PagePathParams) -> Result<C::PagePath> {
        (*self).page(id, params)
    }

    fn parse(&self, content: C::Page) -> Result<C::PageContent> {
        (*self).parse(content)
    }
}

#[derive(Debug)]
pub struct CommandWithSession<'a, 'b, C> {
    command: &'a C,
    session_command: Option<&'b SessionCommand>,
    session: Option<Session>,
}

impl<'a, 'b, C> CommandWithSession<'a, 'b, C> {
    pub fn new(
        command: &'a C,
        session_command: Option<&'b SessionCommand>,
        session: Option<Session>,
    ) -> Self {
        Self {
            command,
            session_command,
            session,
        }
    }
}

impl<C, R> Command for CommandWithSession<'_, '_, C>
where
    C: Command<PagePath = R>,
    R: CommandRequest,
{
    type Page = C::Page;
    type PageContent = C::PageContent;
    type PagePath = C::PagePath;
    type PagePathParams = C::PagePathParams;

    fn page(&self, id: &str, params: C::PagePathParams) -> Result<C::PagePath> {
        let path = self.command.page(id, params)?;
        path.wrap(|request| {
            if let (Some(session_command), Some(session)) = (self.session_command, &self.session) {
                session_command.wrap(request, session.clone())
            } else {
                Ok(request)
            }
        })
    }

    fn parse(&self, content: C::Page) -> Result<C::PageContent> {
        self.command.parse(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hashset, http, runtime};

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

    #[test]
    fn test_wrap() {
        let runtime = crate::runtime::Runtime::new();
        let schema = runtime
            .load(
                r#"--@id: test
--@name: test_schema
--@author: test_author
--@description: test
--@fany_version: 1.0
--@legal_domains: www.example.com

local function search()
end
local function book_info(id)
    return "https://www.example.com"
end
local function chapter()
end
local function toc()
end
local function session()
end
local function session_parse(content)
    return "test"
end
local function wrap(request, session)
    request.url = request.url .. "?session=" .. session
    request.headers = {["User-Agent"] = session}
    return request
end
return {
    search = {page = search, parse = search},
    book_info = {page = book_info, parse = book_info},
    chapter = {page = chapter, parse = chapter},
    toc = {page = toc, parse = toc},
    session = {page = session, parse = session_parse, wrap = wrap},
}"#,
                "test",
            )
            .unwrap();
        let session = schema
            .session
            .as_ref()
            .unwrap()
            .parse("".to_string())
            .unwrap();
        let command =
            CommandWithSession::new(&schema.book_info, schema.session.as_ref(), Some(session));
        let path = command.page("123", ()).unwrap();
        assert_eq!(path.url, "https://www.example.com?session=test");
        assert_eq!(path.headers.get("User-Agent"), Some(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_search() {
        let runtime = crate::runtime::Runtime::new();
        let schema = runtime
            .load(
                r#"--@id: test
--@name: test_schema
--@author: test_author
--@description: test
--@fany_version: 1.0
--@legal_domains: www.example.com

local function search(keyword, page, content)
    if page == 1 then
        return "https://www.example.com"
    end
end
local function search_parse(content)
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
    search = {page = search, parse = search_parse},
    book_info = {page = book_info, parse = book_info},
    chapter = {page = chapter, parse = chapter},
    toc = {page = toc, parse = toc},
    session = {page = session, parse = session, wrap = session},
}"#,
                "test",
            )
            .unwrap();
        let http = HttpClient::new(
            reqwest::Client::new(),
            hashset!["www.example.com".to_string()],
        );
        let mut items = schema.search("keyword", &http, None);
        let first = items
            .next_page()
            .await
            .unwrap()
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        assert_eq!(first.id, "1");
    }

    #[tokio::test]
    async fn test_book_info() {
        let runtime = crate::runtime::Runtime::new();
        let schema = runtime
            .load(
                r#"--@id: test
--@name: test_schema
--@author: test_author
--@description: test
--@fany_version: 1.0
--@legal_domains: www.example.com

local function search()
end
local function book_info(id)
    return "https://www.example.com/" .. id
end
local function book_info_parse(content)
    return {
        title = "title",
        author = "author",
        cover = "cover",
        last_update = "last_update",
        status = "status",
        intro = "intro",
    }
end
local function chapter()
end
local function toc()
end
local function session()
end
return {
    search = {page = search, parse = search},
    book_info = {page = book_info, parse = book_info_parse},
    chapter = {page = chapter, parse = chapter},
    toc = {page = toc, parse = toc},
    session = {page = session, parse = session, wrap = session},
}"#,
                "test",
            )
            .unwrap();
        let http = HttpClient::new(
            reqwest::Client::new(),
            hashset!["www.example.com".to_string()],
        );
        let info = schema.book_info("123", &http, None).await.unwrap();
        assert_eq!(info.title, "title");
        assert_eq!(info.author, "author");
        assert_eq!(info.cover, "cover");
        assert_eq!(info.last_update, "last_update");
        assert_eq!(info.status, "status");
        assert_eq!(info.intro, "intro");
    }
}
