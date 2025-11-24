use std::ops::Deref;
use std::sync::Arc;

use crate::cache::Cache;
use crate::config::AppConfig;
use crate::repo::Repo;

/// `AppState` is a cloneable wrapper around `AppStateInner` using `Arc`.
#[derive(Clone, Debug)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

impl AppState {
    pub fn new(config: AppConfig, repo: Repo, cache: Cache) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                config,
                repo,
                cache,
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
}

/// `AppStateInner` has not to be Clone because `AppState` is the one being cloned.
#[derive(Debug)]
pub struct AppStateInner {
    pub config: AppConfig,
    pub repo: Repo,
    pub cache: Cache,
}
