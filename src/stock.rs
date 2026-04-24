// src/stock.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    value: String,
    expiration: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stock {
    map: HashMap<String, Data>,
}

impl Stock {
    pub fn new() -> Self {
        Stock {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(
            key,
            Data {
                value,
                expiration: None,
            },
        );
    }

    pub fn set_with_expiration(&mut self, key: String, value: String, duration: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let expiration = now + (duration * 1000);

        self.map.insert(
            key,
            Data {
                value,
                expiration: Some(expiration),
            },
        );
    }

    pub fn get(&mut self, key: &String) -> Option<&String> {
        let data = self.map.get(key);

        if data.is_some() {
            if let Some(expiration) = data.unwrap().expiration {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                if now > expiration {
                    self.del(key);
                    return None;
                }
            }
        }
        self.map.get(key).map(|data| &data.value)
    }

    pub fn del(&mut self, key: &String) -> Option<String> {
        self.map.remove(key).map(|data| data.value)
    }

    pub fn save(&self, filename: &String) -> Result<String, String> {
        let json = serde_json::to_string_pretty(&self.map).map_err(|e| e.to_string())?;

        let mut file = File::create("./data/".to_string() + filename).map_err(|e| e.to_string())?;

        file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;

        Ok("OK".into())
    }

    pub fn load(&mut self, filename: &String) -> Result<String, String> {
        let mut file = File::open("./data/".to_string() + filename).map_err(|e| e.to_string())?;
        let mut content = String::new();

        file.read_to_string(&mut content)
            .map_err(|e| e.to_string())?;

        self.map = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        Ok("OK".into())
    }

    pub fn drop(&mut self) {
        self.map.clear();
    }

    pub fn incr(&mut self, key: String) -> Result<i64, String> {
        let current = self.map.get(&key).and_then(|data| {
            if let Some(expiration) = data.expiration {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                if now > expiration {
                    return None;
                }
            }
            data.value.parse::<i64>().ok()
        });

        let new_value = match current {
            Some(v) => v + 1,
            None => 1,
        };

        self.map.insert(
            key,
            Data {
                value: new_value.to_string(),
                expiration: None,
            },
        );

        Ok(new_value)
    }

    pub fn decr(&mut self, key: String) -> Result<i64, String> {
        let current = self.map.get(&key).and_then(|data| {
            if let Some(expiration) = data.expiration {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                if now > expiration {
                    return None;
                }
            }
            data.value.parse::<i64>().ok()
        });

        let new_value = match current {
            Some(v) => v - 1,
            None => -1,
        };

        self.map.insert(
            key,
            Data {
                value: new_value.to_string(),
                expiration: None,
            },
        );

        Ok(new_value)
    }

    pub fn ttl(&mut self, key: String) -> Option<u64> {
        let data = self.map.get(&key)?;

        if let Some(expiration) = data.expiration {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            if now > expiration {
                self.del(&key);
                return None;
            }

            Some(expiration - now)
        } else {
            None
        }
    }

    pub fn exists(&mut self, keys: Vec<String>) -> Vec<(String, bool)> {
        keys.into_iter()
            .map(|key| {
                let exists = self.get(&key).is_some();
                (key, exists)
            })
            .collect()
    }

    pub fn list_keys(&self) -> Vec<String> {
        self.map.keys().cloned().collect()
    }

    pub fn count(&self) -> usize {
        self.map.len()
    }
}
