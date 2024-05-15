use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaesarConfig {
    pub app_environment: String,
    pub app_host: String,
    pub app_port: String,
    pub app_origin: String,
    pub app_relay: String,
    pub rust_log: String,
}

impl Default for CaesarConfig {
    fn default() -> Self {
        CaesarConfig {
            app_environment: "production".to_string(),
            app_host: "localhost".to_string(),
            app_port: "8000".to_string(),
            app_origin: "wss://caesar-transfer-iu.shuttleapp.rs".to_string(),
            app_relay: "localhost:8000".to_string(),
            rust_log: "info".to_string(),
        }
    }
}

lazy_static! {
    pub static ref GLOBAL_CONFIG: CaesarConfig = {
        let cfg: CaesarConfig =
            confy::load("caesar", "caesar").expect("could not find config file");
        cfg
    };
}
