use std::{
    env,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use redis::Connection;

use crate::{cache::Cache, config::CdnConfig, error, info, storage::Storage};

#[derive(Clone)]
pub struct Disconnected;
#[derive(Clone)]
pub struct Connected;

#[derive(Clone)]
pub struct Cdn<State = Disconnected> {
    pub storage: Storage,
    pub cache: Cache,
    pub config: CdnConfig,
    redis: Option<Arc<Mutex<Connection>>>,
    state: PhantomData<State>,
}

impl Cdn<Disconnected> {
    pub fn new(storage: Storage, cache: Cache, config: CdnConfig) -> Self {
        Self {
            storage,
            cache,
            config,
            redis: None,
            state: PhantomData::<Disconnected>,
        }
    }

    pub fn connect(self) -> Cdn<Connected> {
        let redis_host = env::var("REDIS_HOST").unwrap_or("redis://127.0.0.1".to_string());

        let redis_client = redis::Client::open(redis_host).unwrap_or_else(|why| {
            error!("Could not connect to redis: {}", why.to_string());
        });

        let timeout = std::time::Duration::from_secs(10);

        let redis = redis_client
            .get_connection_with_timeout(timeout)
            .unwrap_or_else(|_| {
                error!("Could not connect to redis: timeout");
            });

        info!("Successfully connected to redis");

        Cdn {
            storage: self.storage,
            cache: self.cache,
            config: self.config,
            redis: Some(Arc::new(Mutex::new(redis))),
            state: PhantomData::<Connected>,
        }
    }
}

impl Cdn<Connected> {
    pub fn redis(&self) -> Arc<Mutex<Connection>> {
        self.redis
            .clone()
            .expect("Redis should always be of type Some when Cdn is Connected")
    }
}
