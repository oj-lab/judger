pub struct HttpClient {
    client: reqwest::Client,
    base_url: String,
}

impl HttpClient {
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::new();
        Self { client, base_url }
    }

    pub fn post(&self, path: String) -> reqwest::RequestBuilder {
        self.client.post(format!("{}{}", self.base_url, path))
    }
}
