use crate::db::Db;
use crate::request::Request;
use crate::returns::Return;
use std::sync::Arc;

pub struct App {
    pub db: Arc<Db>,
    pub keys: Vec<String>,
    pub selected: usize,
    pub command_input: String,
    pub history: Vec<String>,
    pub mode: Mode,
    pub tab: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
}

impl App {
    pub fn new(db: Arc<Db>) -> Self {
        Self {
            db,
            keys: Vec::new(),
            selected: 0,
            command_input: String::new(),
            history: Vec::new(),
            mode: Mode::Normal,
            tab: 0,
        }
    }

    pub fn refresh_keys(&mut self) {
        self.keys = self.db.list_all_keys();
        if self.selected >= self.keys.len() && !self.keys.is_empty() {
            self.selected = self.keys.len() - 1;
        }
    }

    pub fn get_value(&self, key: &str) -> Option<String> {
        self.db.get_value(key)
    }

    pub fn get_ttl(&self, key: &str) -> Option<u64> {
        self.db.get_ttl(key)
    }

    pub fn execute(&mut self) {
        if self.command_input.is_empty() {
            return;
        }

        let input = self.command_input.clone();
        self.history.push(format!("> {}", input));

        match Request::parse(&input) {
            Ok(req) => {
                let cmd = req.into_command();
                let result = cmd.execute(&self.db, 0);

                match result {
                    Return::Ok(val) => self.history.push(format!("  ✓ {}", val)),
                    Return::Err(e) => self.history.push(format!("  ✗ {}", e)),
                    Return::NotFound(k) => self.history.push(format!("  ✗ Key '{}' not found", k)),
                    Return::Subscribe(_) => self.history.push("  ✓ Subscribed".to_string()),
                    Return::Unsubscribe => self.history.push("  ✓ Unsubscribed".to_string()),
                }

                self.refresh_keys();
            }
            Err(e) => self.history.push(format!("  ✗ {}", e)),
        }

        self.command_input.clear();
    }

    pub fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn down(&mut self) {
        if self.selected < self.keys.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn next_tab(&mut self) {
        self.tab = (self.tab + 1) % 3;
    }

    pub fn prev_tab(&mut self) {
        self.tab = (self.tab + 2) % 3;
    }
}
