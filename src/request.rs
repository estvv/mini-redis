use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Request {
    GET(String),
    SET { key: String, value: String, expiration: Option<u64> },
    DEL(String),
    INCR(String),
    DECR(String),
    SAVE(String),
    LOAD(String),
    DROP(),
    PUB { channel: String, message: String },
    SUB(String),
    UNSUB(String),
    TTL(String),
    EXISTS(Vec<String>),
}

const GET_MIN_ARGS: usize = 2;
const GET_MAX_ARGS: usize = 2;

const SET_MIN_ARGS: usize = 3;

const DEL_MIN_ARGS: usize = 2;
const DEL_MAX_ARGS: usize = 2;

const INCR_MIN_ARGS: usize = 2;
const INCR_MAX_ARGS: usize = 2;

const DECR_MIN_ARGS: usize = 2;
const DECR_MAX_ARGS: usize = 2;

const SAVE_ARGS: usize = 2;
const LOAD_ARGS: usize = 2;

const PUB_MIN_ARGS: usize = 3;

const SUB_MIN_ARGS: usize = 2;
const SUB_MAX_ARGS: usize = 2;

const TTL_MIN_ARGS: usize = 2;
const TTL_MAX_ARGS: usize = 2;

const EXISTS_MIN_ARGS: usize = 2;

impl Request {
    pub fn parse(input: &str) -> Result<Self, String> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        if parts.is_empty() {
            return Err("Invalid request".into());
        }

        match parts[0].to_uppercase().as_str() {
            "GET" => Request::get_callback(parts),
            "SET" => Request::set_callback(parts),
            "DEL" => Request::del_callback(parts),
            "INCR" => Request::incr_callback(parts),
            "DECR" => Request::decr_callback(parts),
            "SAVE" => Request::save_callback(parts),
            "LOAD" => Request::load_callback(parts),
            "DROP" => Request::drop_callback(),
            "PUB" => Request::pub_callback(parts),
            "SUB" => Request::sub_callback(parts),
            "UNSUB" => Request::unsub_callback(parts),
            "TTL" => Request::ttl_callback(parts),
            "EXISTS" => Request::exists_callback(parts),
            _ => Err("Unknown command".to_string()),
        }
    }

    pub fn get_callback(parts: Vec<&str>) -> Result<Self, String> {
        if parts.len() < GET_MIN_ARGS {
            Err("Invalid GET request. Too few arguments".to_string())
        } else if parts.len() > GET_MAX_ARGS {
            Err("Invalid GET request. Too many arguments".to_string())
        } else {
            Ok(Request::GET(parts[1].to_string()))
        }
    }

    pub fn set_callback(parts: Vec<&str>) -> Result<Self, String> {
        if parts.len() < SET_MIN_ARGS {
            return Err("Invalid SET request. Too few arguments".to_string());
        }

        let exp_index = parts.iter().position(|p| p.to_uppercase() == "EXP");

        let (key, value, expiration) = match exp_index {
            Some(idx) => {
                if idx < 2 {
                    return Err(
                        "Invalid SET request. EXP cannot be before key and value".to_string()
                    );
                }
                if idx + 1 >= parts.len() {
                    return Err("Invalid SET request. EXP requires a value".to_string());
                }

                let exp: u64 = parts[idx + 1].parse().map_err(|_| "Invalid expiration value".to_string())?;
                let value = parts[2..idx].join(" ");

                (parts[1].to_string(), value, Some(exp))
            }
            None => {
                let value = parts[2..].join(" ");
                (parts[1].to_string(), value, None)
            }
        };

        Ok(Request::SET {
            key,
            value,
            expiration,
        })
    }

    pub fn del_callback(parts: Vec<&str>) -> Result<Self, String> {
        if parts.len() < DEL_MIN_ARGS {
            Err("Invalid DEL request. Too few arguments".to_string())
        } else if parts.len() > DEL_MAX_ARGS {
            Err("Invalid DEL request. Too many arguments".to_string())
        } else {
            Ok(Request::DEL(parts[1].to_string()))
        }
    }

    pub fn incr_callback(parts: Vec<&str>) -> Result<Self, String> {
        if parts.len() < INCR_MIN_ARGS {
            Err("Invalid INCR request. Too few arguments".to_string())
        } else if parts.len() > INCR_MAX_ARGS {
            Err("Invalid INCR request. Too many arguments".to_string())
        } else {
            Ok(Request::INCR(parts[1].to_string()))
        }
    }

    pub fn decr_callback(parts: Vec<&str>) -> Result<Self, String> {
        if parts.len() < DECR_MIN_ARGS {
            Err("Invalid DECR request. Too few arguments".to_string())
        } else if parts.len() > DECR_MAX_ARGS {
            Err("Invalid DECR request. Too many arguments".to_string())
        } else {
            Ok(Request::DECR(parts[1].to_string()))
        }
    }

    pub fn save_callback(parts: Vec<&str>) -> Result<Self, String> {
        if parts.len() < SAVE_ARGS {
            Err("Invalid SAVE request. Need filename argument.".to_string())
        } else if parts.len() > SAVE_ARGS {
            Err("Invalid SAVE request. Too many arguments".to_string())
        } else {
            if !parts[1].ends_with(".json") {
                return Err("Invalid SAVE request. Filename must end with .json".to_string());
            }
            Ok(Request::SAVE(parts[1].to_string()))
        }
    }

    pub fn load_callback(parts: Vec<&str>) -> Result<Self, String> {
        if parts.len() < LOAD_ARGS {
            Err("Invalid LOAD request. Need filename argument.".to_string())
        } else if parts.len() > LOAD_ARGS {
            Err("Invalid LOAD request. Too many arguments".to_string())
        } else {
            if !parts[1].ends_with(".json") {
                return Err("Invalid LOAD request. Filename must end with .json".to_string());
            }
            Ok(Request::LOAD(parts[1].to_string()))
        }
    }

    pub fn drop_callback() -> Result<Self, String> {
        Ok(Request::DROP())
    }

    pub fn pub_callback(parts: Vec<&str>) -> Result<Self, String> {
        if parts.len() < PUB_MIN_ARGS {
            Err("Invalid PUB request. Need channel and message arguments.".to_string())
        } else {
            let channel = parts[1].to_string();
            let message = parts[2..].join(" ");

            Ok(Request::PUB { channel, message })
        }
    }

    pub fn sub_callback(parts: Vec<&str>) -> Result<Self, String> {
        if parts.len() < SUB_MIN_ARGS {
            Err("Invalid SUB request. Need channel argument.".to_string())
        } else if parts.len() > SUB_MAX_ARGS {
            Err("Invalid SUB request. Too many arguments.".to_string())
        } else {
            Ok(Request::SUB(parts[1].to_string()))
        }
    }

    pub fn unsub_callback(parts: Vec<&str>) -> Result<Self, String> {
        if parts.len() < SUB_MIN_ARGS {
            Err("Invalid UNSUB request. Need channel argument.".to_string())
        } else if parts.len() > SUB_MAX_ARGS {
            Err("Invalid UNSUB request. Too many arguments.".to_string())
        } else {
            Ok(Request::UNSUB(parts[1].to_string()))
        }
    }

    pub fn ttl_callback(parts: Vec<&str>) -> Result<Self, String> {
        if parts.len() < TTL_MIN_ARGS {
            Err("Invalid TTL request. Need key argument.".to_string())
        } else if parts.len() > TTL_MAX_ARGS {
            Err("Invalid TTL request. Too many arguments.".to_string())
        } else {
            Ok(Request::TTL(parts[1].to_string()))
        }
    }

    pub fn exists_callback(parts: Vec<&str>) -> Result<Self, String> {
        if parts.len() < EXISTS_MIN_ARGS {
            Err("Invalid EXISTS request. Need key argument.".to_string())
        } else {
            Ok(Request::EXISTS(parts[1..].iter().map(|s| s.to_string()).collect()))
        }
    }
}
