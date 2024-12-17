use crate::Result;
use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct HttpClient;

impl HttpClient {
    pub async fn request(&self, _request: &HttpRequest) -> Result<String> {
        unimplemented!()
    }
}
