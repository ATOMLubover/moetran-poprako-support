use redis::{AsyncTypedCommands as _, RedisResult};

use crate::cache::Cache;

pub async fn fetch_add_projset_serial(team_id: &str, cache: &Cache) -> RedisResult<i32> {
    let mut conn = cache.client().get_multiplexed_async_connection().await?;

    let new_serial = conn
        .incr(format!("team:{}:projset_serial", team_id), 1)
        .await?;

    Ok(new_serial as i32)
}

pub async fn init_projset_index(projset_id: &str, cache: &Cache) -> RedisResult<bool> {
    let mut conn = cache.client().get_multiplexed_async_connection().await?;

    // Initialize the projset index counter to 0.
    let successful = conn
        .set_nx(format!("projset:{}:index", projset_id), 0)
        .await?;

    Ok(successful)
}
