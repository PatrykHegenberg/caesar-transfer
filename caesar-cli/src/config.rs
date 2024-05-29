use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

/// Represents the configuration settings for the Caesar application.
///
/// This struct is used to store the configuration settings for the application,
/// such as the environment, host, port, origin, and logging level.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaesarConfig {
    /// The environment in which the application is running.
    ///
    /// Possible values are "production", "staging", or "development".
    pub app_environment: String,

    /// The host on which the application is running.
    ///
    /// This is typically an IP address or a hostname.
    pub app_host: String,

    /// The port on which the application is listening.
    ///
    /// This is typically a string representation of a port number.
    pub app_port: String,

    /// The origin of the application.
    ///
    /// This is typically a URL that specifies the protocol, hostname, and port.
    pub app_origin: String,

    /// The relay endpoint of the application.
    ///
    /// This is typically a combination of a hostname and port.
    pub app_relay: String,

    /// The logging level for the application.
    ///
    /// This is typically a string representation of a logging level, such as "info",
    /// "debug", or "error".
    pub rust_log: String,
}


/// The default configuration values for the Caesar application.
///
/// These values are used when loading the configuration file fails.
/// The default configuration is suitable for running the application in a production environment.
impl Default for CaesarConfig {
    /// Returns a new `CaesarConfig` instance with default values.
    ///
    /// # Returns
    ///
    /// A new `CaesarConfig` instance with the following default values:
    ///
    /// - `app_environment`: "production"
    /// - `app_host`: "0.0.0.0"
    /// - `app_port`: "8000"
    /// - `app_origin`: "wss://caesar-transfer-iu.shuttleapp.rs"
    /// - `app_relay`: "0.0.0.0:8000"
    /// - `rust_log`: "info"
    fn default() -> Self {
        CaesarConfig {
            app_environment: "production".to_string(),  // The environment in which the application is running.
            app_host: "0.0.0.0".to_string(),           // The host on which the application is running.
            app_port: "8000".to_string(),              // The port on which the application is listening.
            app_origin: "wss://caesar-transfer-iu.shuttleapp.rs".to_string(),  // The origin of the application.
            app_relay: "0.0.0.0:8000".to_string(),     // The relay endpoint of the application.
            rust_log: "info".to_string(),              // The logging level for the application.
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
