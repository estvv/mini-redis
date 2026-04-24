// src/returns.rs

use tokio::sync::broadcast;

#[derive(Debug)]
pub enum Return {
    Ok(String),
    Err(String),
    NotFound(String),
    Subscribe(broadcast::Receiver<String>),
    Unsubscribe
}

impl PartialEq for Return {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Return::Ok(a), Return::Ok(b)) => a == b,
            (Return::Err(a), Return::Err(b)) => a == b,
            (Return::NotFound(a), Return::NotFound(b)) => a == b,
            (Return::Unsubscribe, Return::Unsubscribe) => true,
            (Return::Subscribe(_), Return::Subscribe(_)) => false,
            _ => false,
        }
    }
}
