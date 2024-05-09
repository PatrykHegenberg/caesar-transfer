use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaesarConfig {
    pub app_environment: String,
    pub app_host: String,
    pub app_port: String,
    pub app_origin: String,
    pub app_relay: String,
    pub rust_log: String,
}

lazy_static! {
    pub static ref GLOBAL_CONFIG: CaesarConfig = {
        let cfg: CaesarConfig =
            confy::load("caesar", "caesar").expect("could not find config file");
        cfg
    };
}
