pub mod client;
pub mod http_client;

use crate::{receiver::client as receiver, sender::util::replace_protocol};

use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue},
};
use tracing::{debug, error};

pub async fn start_receiver(relay: &str, name: &str) {
    let http_url = replace_protocol(relay);
    let res = http_client::download_info(http_url.as_str(), name)
        .await
        .unwrap();
    debug!("Got room_id from Server: {:?}", res);
    let res_ip = String::from("ws://") + res.ip.as_str() + ":9000";

    if let Err(local_err) = start_ws_com(res_ip.as_str(), res.local_room_id.as_str()).await {
        debug!("Failed to connect local: {local_err}");
        if let Err(relay_err) = start_ws_com(relay, res.relay_room_id.as_str()).await {
            debug!("Failed to connect remote: {relay_err}");
        }
    }
    let success = http_client::download_success(http_url.as_str(), name).await;
    match success {
        Ok(()) => debug!("Success"),
        Err(e) => error!("Error: {e:?}"),
    };
}

pub async fn start_ws_com(relay: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = String::from(relay) + "/ws";
    let Ok(mut request) = url.into_client_request() else {
        println!("Error: Failed to create request.");
        return Err("Failed to create request".into());
    };

    request
        .headers_mut()
        .insert("Origin", HeaderValue::from_str(relay).unwrap());

    println!("Attempting to connect...");

    let _ = match tokio::time::timeout(std::time::Duration::from_secs(5), connect_async(request))
        .await
    {
        Ok(Ok((socket, _))) => {
            receiver::start(socket, name).await;
            Ok(())
        }
        Ok(Err(e)) => {
            error!("Error: Failed to connect: {e:?}");
            Err(Box::new(e))
        }
        Err(e) => {
            error!("Error: Timeout reached for local connection attempt");
            Err(Box::new(e))
        }?,
    };
    Ok(())
}
