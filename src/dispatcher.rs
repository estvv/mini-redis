// src/dispatcher.rs

use crate::channel_manager::ChannelManager;
use crate::request::Request;
use crate::returns::Return;
use crate::stock::Stock;
use std::collections::HashMap;

pub struct Dispatcher {
    stock: Stock,
    channel_manager: ChannelManager,
    client_subscriptions: HashMap<u64, String>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Dispatcher {
            stock: Stock::new(),
            channel_manager: ChannelManager::new(),
            client_subscriptions: HashMap::new(),
        }
    }

    pub fn dispatch(&mut self, request: Request, client_id: u64) -> Return {
        match request {
            Request::GET(key) => self.get(key),
            Request::SET { key, value, expiration } => self.set(key, value, expiration),
            Request::DEL(key) => self.del(key),
            Request::INCR(key) => self.incr(key),
            Request::DECR(key) => self.decr(key),
            Request::SAVE(filename) => self.save(filename),
            Request::LOAD(filename) => self.load(filename),
            Request::DROP() => self.drop(),
            Request::PUB { channel, message } => self.publish(channel, message),
            Request::SUB(channel) => self.subscribe(channel, client_id),
            Request::UNSUB(channel) => self.unsubscribe(channel, client_id),
            Request::TTL(key) => self.ttl(key),
            Request::EXISTS(key) => self.exists(key),
        }
    }

    pub fn get(&mut self, key: String) -> Return {
        if self.stock.get(&key).is_none() {
            return Return::NotFound(key.to_string());
        }

        Return::Ok(self.stock.get(&key).unwrap().to_string())
    }

    pub fn set(&mut self, key: String, value: String, expiration: Option<u64>) -> Return {
        if expiration.is_none() {
            self.stock.set(key, value);
            return Return::Ok("OK".into());
        }

        self.stock
            .set_with_expiration(key, value, expiration.unwrap());
        Return::Ok("OK".into())
    }

    pub fn del(&mut self, key: String) -> Return {
        if !self.stock.get(&key).is_some() {
            return Return::NotFound(key.to_string());
        }

        self.stock.del(&key);
        Return::Ok("OK".into())
    }

    pub fn incr(&mut self, key: String) -> Return {
        match self.stock.incr(key) {
            Ok(value) => Return::Ok(value.to_string()),
            Err(e) => Return::Err(e),
        }
    }

    pub fn decr(&mut self, key: String) -> Return {
        match self.stock.decr(key) {
            Ok(value) => Return::Ok(value.to_string()),
            Err(e) => Return::Err(e),
        }
    }

    pub fn save(&mut self, filename: String) -> Return {
        match self.stock.save(&filename) {
            Ok(msg) => Return::Ok(msg),
            Err(e) => Return::Err(e),
        }
    }

    pub fn load(&mut self, filename: String) -> Return {
        match self.stock.load(&filename) {
            Ok(msg) => Return::Ok(msg),
            Err(e) => Return::Err(e),
        }
    }

    pub fn drop(&mut self) -> Return {
        self.stock.drop();
        Return::Ok("OK".into())
    }

    pub fn publish(&mut self, channel: String, message: String) -> Return {
        match self.channel_manager.publish(&channel, &message) {
            Ok(count) => Return::Ok(format!("Published to {} subscriber(s)", count)),
            Err(e) => Return::Err(e),
        }
    }

    pub fn subscribe(&mut self, channel: String, client_id: u64) -> Return {
        self.client_subscriptions.insert(client_id, channel.clone());
        let receiver = self.channel_manager.subscribe(channel);

        Return::Subscribe(receiver)
    }

    pub fn unsubscribe(&mut self, channel: String, client_id: u64) -> Return {
        if !self.channel_manager.channel_exists(&channel) {
            return Return::Err(format!("Channel '{}' does not exist", channel));
        }

        match self.client_subscriptions.get(&client_id) {
            Some(subscribed_channel) => {
                if subscribed_channel != &channel {
                    return Return::Err(format!("Not subscribed to channel '{}'", channel));
                }
            }
            None => {
                return Return::Err("Not subscribed to any channel".to_string());
            }
        }

        self.client_subscriptions.remove(&client_id);
        Return::Unsubscribe
    }

    pub fn cleanup_client(&mut self, client_id: u64) {
        self.client_subscriptions.remove(&client_id);
    }

    pub fn ttl(&mut self, key: String) -> Return {
        if self.stock.get(&key).is_none() {
            return Return::NotFound(key.to_string());
        }

        match self.stock.ttl(key.clone()) {
            Some(ttl) => Return::Ok(ttl.to_string() + "ms"),
            None => Return::Ok("None".to_string()),
        }
    }

    pub fn exists(&mut self, keys: Vec<String>) -> Return {
        let results = self.stock.exists(keys);
        let formatted = results.iter().map(|(k, exists)| format!("{} -> {}", k, exists)).collect::<Vec<_>>().join("\r\n");

        Return::Ok(formatted)
    }
}
