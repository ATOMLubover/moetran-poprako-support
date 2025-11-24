use redis::RedisResult;

use crate::cache::Cache;

pub async fn fetch_add_projset_index(projset_id: &str, cache: &Cache) -> RedisResult<i32> {
    // If the index counter does not exist, return error instead of creating a new one.
    // The index counter should be created when the projset is created.
    const SCRIPT: &str = r#"
        -- KEY[1] = projset index key, like "projset:{projset_id}:index"
        -- return: > 0 as normal, -1 as error 

        if redis.call("EXISTS", KEYS[1]) == 0 then
            -- No index counter exists, return an -1 as error.
            return -1
        end

        -- Increment and return the new index.
        return redis.call("INCR", KEYS[1])
    "#;

    let mut conn = cache.client().get_multiplexed_async_connection().await?;

    let new_index: i32 = redis::Script::new(SCRIPT)
        .key(format!("projset:{}:index", projset_id))
        .invoke_async(&mut conn)
        .await?;

    Ok(new_index)
}
