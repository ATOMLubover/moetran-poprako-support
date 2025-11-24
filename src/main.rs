#[tokio::main]
async fn main() -> anyhow::Result<()> {
    moetran_support::run().await
}
