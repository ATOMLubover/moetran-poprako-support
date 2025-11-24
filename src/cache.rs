use redis::{AsyncTypedCommands, Client, RedisResult};

#[derive(Debug)]
pub struct Cache {
    client: Client,
}

impl Cache {
    pub fn new() -> anyhow::Result<Self> {
        let redis_url = std::env::var("REDIS_URL")
            .map_err(|err| anyhow::anyhow!("Error when acquiring REDIS_URL: {}", err))?;

        let client = Client::open(redis_url.as_str())
            .map_err(|err| anyhow::anyhow!("Error when connecting to Redis: {}", err))?;

        Ok(Self { client })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub async fn ping(&self) -> RedisResult<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        conn.ping().await?;

        Ok(())
    }
}
