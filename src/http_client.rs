use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, Request, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpStream,
};

pub async fn send_request(
    url: &str,
    method: &str,
    body: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = url.parse::<hyper::Uri>()?;
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);
    let address = format!("{}:{}", host, port);
    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            eprintln!("Connection failed: {:?}", err);
        }
    });

    let authority = url.authority().unwrap().clone();
    let send_body = match body {
        Some(body_str) => Full::<Bytes>::from(Bytes::from(body_str)),
        None => Full::<Bytes>::from(Bytes::from("")),
    };

    let req = Request::builder()
        .method(method)
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .body(send_body)?;

    let mut res = sender.send_request(req).await?;

    println!("Response status: {}", res.status());

    if res.status() == StatusCode::OK {
        while let Some(next) = res.frame().await {
            let frame = next?;
            if let Some(chunk) = frame.data_ref() {
                io::stdout().write_all(chunk).await?;
            }
        }
    }
    Ok(())
}

