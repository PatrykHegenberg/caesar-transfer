use crate::relay::server;
use crate::{
    receiver::client as receiver,
    sender::{client as sender, server::serf_file},
};
use clap::{Parser, Subcommand};
use tracing::{debug, error, info};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Send files to the receiver or relay server
    Send {
        /// Address of the relay server. Accepted formats are: 127.0.0.1:8080, [::1]:8080, example.com
        #[arg(short, long)]
        relay: Option<String>,
        /// Path to file(s)
        #[arg(value_name = "FILE")]
        file: Option<String>,
    },
    /// Receives Files from the sender with the matching password
    Receive {
        /// Address of the relay server. Accepted formats are: 127.0.0.1:8080, [::1]:8080, example.com
        #[arg(short, long)]
        relay: Option<String>,

        /// Overwrite existing Files
        #[arg(short, long)]
        overwrite: bool,

        /// Name of Transfer to download files
        #[arg(value_name = "Transfer_Name")]
        name: Option<String>,
    },
    /// Start a relay server
    Serve {
        /// Port to run the relay server on
        #[arg(short, long)]
        port: Option<i32>,
        /// The Listen address to run the relay server on
        #[arg(short, long)]
        listen_address: Option<String>,
    },
    /// Work with your configuration
    Config {
        /// Show path to config file
        #[arg(short, long)]
        path: bool,

        /// View configured Options
        #[arg(short, long)]
        show: bool,

        /// Edit the config file
        #[arg(short, long)]
        edit: bool,

        /// Reset changed config
        #[arg(short, long)]
        reset: bool,
    },
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}

impl Args {
    pub fn new() -> Self {
        Self::parse()
    }
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("args: {:#?}", self);
        match &self.command {
            Some(Commands::Send { relay, file }) => {
                let _ = match sender::send_info(
                    relay.as_deref().unwrap_or("http://0.0.0.0:8000"),
                    file.as_deref().unwrap_or("test.txt"),
                )
                .await
                {
                    Ok(name) => {
                        println!("Transfer name: {}", name);
                        serf_file(file.as_ref().unwrap()).await;
                        Ok(())
                    }
                    Err(err) => Err(err),
                };
            }
            Some(Commands::Receive {
                relay,
                overwrite,
                name,
            }) => {
                let response = receiver::download_info(
                    relay.as_deref().unwrap_or("http://0.0.0.0:8000"),
                    name.as_deref().unwrap_or("None"),
                )
                .await;
                match response {
                    Ok(res) => {
                        debug!("The response is: {:#?}", res);
                        let reachable = receiver::ping_sender(&res.body.ip).await;
                        match reachable {
                            Ok(_) => match receiver::download_file(&res, overwrite).await {
                                Ok(_) => {
                                    info!("Download complete");
                                    receiver::signal_success_relay(
                                        relay.as_deref().unwrap_or("http://0.0.0.0:8000"),
                                        name.as_deref().unwrap_or("None"),
                                    )
                                    .await?;
                                    let _ =
                                        match receiver::signal_success_sender(&res.body.ip).await {
                                            Ok(_) => Ok(()),
                                            Err(err) => Err(err),
                                        };
                                }
                                Err(err) => error!(err),
                            },
                            Err(err) => error!("Error: {:#?}", err),
                        }
                    }
                    Err(err) => error!("Error: {:#?}", err),
                }
            }
            Some(Commands::Serve {
                port,
                listen_address,
            }) => {
                server::start_server(port.as_ref(), listen_address.as_ref()).await;
            }
            Some(Commands::Config {
                path: _,
                show: _,
                edit: _,
                reset: _,
            }) => {}
            None => {}
        }
        Ok(())
    }
}
