use reqwest::Client;

#[derive(Debug, Clone)]
pub struct Crawler {
    client: Client,
}

impl Crawler {
    pub fn new() -> Self {
        Crawler {
            client: Client::new(),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
