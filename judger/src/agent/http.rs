use reqwest::Url;

pub struct HttpClient {
    client: reqwest::Client,
    base_url: String,
}

impl HttpClient {
    pub fn new(base_url: String, internal_token: String) -> Self {
        let client = reqwest::Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {}", internal_token))
                        .unwrap(),
                );
                headers
            })
            .build()
            .unwrap();
        Self { client, base_url }
    }

    pub fn post(&self, path: String) -> Result<reqwest::RequestBuilder, anyhow::Error> {
        let url = Url::parse(&format!("{}{}", self.base_url, path))?;
        Ok(self.client.post(url))
    }

    pub fn put(&self, path: String) -> Result<reqwest::RequestBuilder, anyhow::Error> {
        let url = Url::parse(&format!("{}{}", self.base_url, path))?;
        Ok(self.client.put(url))
    }
}
