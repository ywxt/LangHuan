use serde::{Deserialize, Serialize};

use crate::{Result, SchemaError, SchemaResult, StdResult};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Method(reqwest::Method);

impl Method {
    pub fn from_bytes(s: &[u8]) -> SchemaResult<Self> {
        reqwest::Method::from_bytes(s)
            .map(Method)
            .map_err(|_| SchemaError::InvalidRequest(format!("invalid method: {:?}", s)))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub(self) fn into_inner(self) -> reqwest::Method {
        self.0
    }
}

impl AsRef<str> for Method {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Serialize for Method {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Method {
    fn deserialize<D>(deserializer: D) -> StdResult<Method, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Method::from_bytes(s.as_bytes()).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRequest {
    pub url: String,
    #[serde(default)]
    pub method: Method,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct HttpClient {
    client: reqwest::Client,
    allowed_domains: HashSet<String>,
}

impl HttpClient {
    pub fn new(client: reqwest::Client, allowed_domains: HashSet<String>) -> Self {
        Self {
            client,
            allowed_domains,
        }
    }
    pub async fn request(&self, request: HttpRequest) -> Result<String> {
        let url = reqwest::Url::parse(&request.url)
            .map_err(|e| SchemaError::InvalidUrl(format!("{} for {}", e, request.url)))?;
        if let Some(domain) = url.domain() {
            if !self.allowed_domains.contains(domain) {
                Err(SchemaError::NotAllowedDomain(domain.to_string()))?
            } else {
                let mut builder = self.client.request(request.method.into_inner(), url);
                for (key, value) in request.headers.into_iter() {
                    builder = builder.header(key, value);
                }
                if !request.body.is_empty() {
                    builder = builder.body(request.body);
                }
                let response = builder.send().await?;
                let text = response.text().await?;
                Ok(text)
            }
        } else {
            Err(SchemaError::InvalidUrl(format!(
                "no domain in {}",
                request.url
            )))?
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Error;

    use super::*;

    #[test]
    fn test_method() {
        let method = Method::from_bytes(b"GET").unwrap();
        assert_eq!(method.as_str(), "GET");
        assert_eq!(method.into_inner(), reqwest::Method::GET);
    }

    #[tokio::test]
    async fn test_http_request() {
        let request = HttpRequest {
            url: "http://bilibili.com".to_string(),
            method: Method::from_bytes(b"GET").unwrap(),
            headers: HashMap::new(),
            body: Vec::new(),
        };
        let mut allowed_domains = HashSet::new();
        allowed_domains.insert("bilibili.com".to_string());
        let client = HttpClient {
            client: reqwest::Client::new(),
            allowed_domains,
        };
        let text = client.request(request).await.unwrap();
        assert!(text.contains("bilibili"));

        let request = HttpRequest {
            url: "http://baidu.com".to_string(),
            method: Method::from_bytes(b"GET").unwrap(),
            headers: HashMap::new(),
            body: Vec::new(),
        };
        assert!(matches!(
            client.request(request).await,
            Err(Error::SchemaError(SchemaError::NotAllowedDomain(_)))
        ));
    }
}
