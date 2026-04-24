use crate::channel_manager::ChannelManager;
use crate::returns::Return;
use crate::stock::Stock;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Db {
    inner: Arc<Mutex<DbInner>>,
}

struct DbInner {
    stock: Stock,
    channel_manager: ChannelManager,
    client_subscriptions: HashMap<u64, String>,
}

impl Db {
    pub fn new() -> Self {
        Db {
            inner: Arc::new(Mutex::new(DbInner {
                stock: Stock::new(),
                channel_manager: ChannelManager::new(),
                client_subscriptions: HashMap::new(),
            })),
        }
    }

    pub fn get(&self, key: String) -> Return {
        let mut inner = self.inner.lock().unwrap();
        if inner.stock.get(&key).is_none() {
            return Return::NotFound(key.to_string());
        }

        Return::Ok(inner.stock.get(&key).unwrap().to_string())
    }

    pub fn set(&self, key: String, value: String, expiration: Option<u64>) -> Return {
        let mut inner = self.inner.lock().unwrap();
        if expiration.is_none() {
            inner.stock.set(key, value);
            return Return::Ok("OK".into());
        }

        inner
            .stock
            .set_with_expiration(key, value, expiration.unwrap());
        Return::Ok("OK".into())
    }

    pub fn del(&self, key: String) -> Return {
        let mut inner = self.inner.lock().unwrap();
        if !inner.stock.get(&key).is_some() {
            return Return::NotFound(key.to_string());
        }

        inner.stock.del(&key);
        Return::Ok("OK".into())
    }

    pub fn incr(&self, key: String) -> Return {
        let mut inner = self.inner.lock().unwrap();
        match inner.stock.incr(key) {
            Ok(value) => Return::Ok(value.to_string()),
            Err(e) => Return::Err(e),
        }
    }

    pub fn decr(&self, key: String) -> Return {
        let mut inner = self.inner.lock().unwrap();
        match inner.stock.decr(key) {
            Ok(value) => Return::Ok(value.to_string()),
            Err(e) => Return::Err(e),
        }
    }

    pub fn save(&self, filename: String) -> Return {
        let inner = self.inner.lock().unwrap();
        match inner.stock.save(&filename) {
            Ok(msg) => Return::Ok(msg),
            Err(e) => Return::Err(e),
        }
    }

    pub fn load(&self, filename: String) -> Return {
        let mut inner = self.inner.lock().unwrap();
        match inner.stock.load(&filename) {
            Ok(msg) => Return::Ok(msg),
            Err(e) => Return::Err(e),
        }
    }

    pub fn clear(&self) -> Return {
        let mut inner = self.inner.lock().unwrap();
        inner.stock.drop();
        Return::Ok("OK".into())
    }

    pub fn publish(&self, channel: String, message: String) -> Return {
        let inner = self.inner.lock().unwrap();
        match inner.channel_manager.publish(&channel, &message) {
            Ok(count) => Return::Ok(format!("Published to {} subscriber(s)", count)),
            Err(e) => Return::Err(e),
        }
    }

    pub fn subscribe(&self, channel: String, client_id: u64) -> Return {
        let mut inner = self.inner.lock().unwrap();
        inner
            .client_subscriptions
            .insert(client_id, channel.clone());
        let receiver = inner.channel_manager.subscribe(channel);

        Return::Subscribe(receiver)
    }

    pub fn unsubscribe(&self, channel: String, client_id: u64) -> Return {
        let mut inner = self.inner.lock().unwrap();
        if !inner.channel_manager.channel_exists(&channel) {
            return Return::Err(format!("Channel '{}' does not exist", channel));
        }

        match inner.client_subscriptions.get(&client_id) {
            Some(subscribed_channel) => {
                if subscribed_channel != &channel {
                    return Return::Err(format!("Not subscribed to channel '{}'", channel));
                }
            }
            None => {
                return Return::Err("Not subscribed to any channel".to_string());
            }
        }

        inner.client_subscriptions.remove(&client_id);
        Return::Unsubscribe
    }

    pub fn cleanup_client(&self, client_id: u64) {
        let mut inner = self.inner.lock().unwrap();
        inner.client_subscriptions.remove(&client_id);
    }

    pub fn ttl(&self, key: String) -> Return {
        let mut inner = self.inner.lock().unwrap();
        if inner.stock.get(&key).is_none() {
            return Return::NotFound(key.to_string());
        }

        match inner.stock.ttl(key.clone()) {
            Some(ttl) => Return::Ok(ttl.to_string() + "ms"),
            None => Return::Ok("None".to_string()),
        }
    }

    pub fn exists(&self, keys: Vec<String>) -> Return {
        let mut inner = self.inner.lock().unwrap();
        let results = inner.stock.exists(keys);
        let formatted = results
            .iter()
            .map(|(k, exists)| format!("{} -> {}", k, exists))
            .collect::<Vec<_>>()
            .join("\r\n");

        Return::Ok(formatted)
    }

    pub fn list_all_keys(&self) -> Vec<String> {
        self.inner.lock().unwrap().stock.list_keys()
    }

    pub fn get_value(&self, key: &str) -> Option<String> {
        self.inner
            .lock()
            .unwrap()
            .stock
            .get(&key.to_string())
            .map(|s| s.to_string())
    }

    pub fn get_ttl(&self, key: &str) -> Option<u64> {
        self.inner.lock().unwrap().stock.ttl(key.to_string())
    }

    pub fn key_count(&self) -> usize {
        self.inner.lock().unwrap().stock.count()
    }
}

impl Clone for Db {
    fn clone(&self) -> Self {
        Db {
            inner: Arc::clone(&self.inner),
        }
    }
}
