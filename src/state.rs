use std::sync::Arc;

use crate::cache::Cache;
use crate::config::AppConfig;
use crate::jwt::JwtCodec;
use crate::repo::Repo;

/// `AppState` is a cloneable wrapper around `AppStateInner` using `Arc`.
#[derive(Clone, Debug)]
pub(crate) struct AppState {
    inner: Arc<Inner>,
}

impl AppState {
    pub fn new(config: AppConfig, repo: Repo, cache: Cache, jwt_codec: JwtCodec) -> Self {
        Self {
            inner: Arc::new(Inner {
                config,
                repo,
                cache,
                jwt_codec,
            }),
        }
    }

    pub fn config(&self) -> &AppConfig {
        &self.inner.config
    }

    pub fn repo(&self) -> &Repo {
        &self.inner.repo
    }

    pub fn cache(&self) -> &Cache {
        &self.inner.cache
    }

    pub fn jwt_codec(&self) -> &JwtCodec {
        &self.inner.jwt_codec
    }
}

/// `AppStateInner` has not to be Clone because `AppState` is the one being cloned.
#[derive(Debug)]
pub struct Inner {
    config: AppConfig,
    repo: Repo,
    cache: Cache,
    jwt_codec: JwtCodec,
}
