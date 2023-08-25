use std::{
    borrow::BorrowMut,
    error::Error,
    io::{self, Read, Write},
    net::SocketAddr,
};

use hyper::{
    body::Body, header::CONTENT_ENCODING, service::service_fn, Client, Request, Response, Server,
};
use thiserror::Error;
use tower::make::Shared;

// ** ####################################################################################################
// ** Suitable Error struct
// ** ####################################################################################################
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("data store disconnected")]
    HyperError(#[from] hyper::Error),
    #[error("data store disconnected")]
    HyperHttpError(#[from] hyper::http::Error),
}

// ** ####################################################################################################
// ** Main function
// ** ####################################################################################################
#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let make_service = Shared::new(service_fn(handle));
    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        println!("error: {}", e);
    }
}

// ** ####################################################################################################
// ** Handling proxy block
// ** ####################################################################################################
async fn handle(request: Request<Body>) -> Result<Response<Body>, ProxyError> {
    let request = store_request(request).await?;
    let response = get_response(request).await?;
    let response = store_response(response).await?;
    Ok(response)
}

async fn get_response(request: Request<Body>) -> Result<Response<Body>, ProxyError> {
    let client = Client::new();
    Ok(client.request(request).await?)
}

async fn store_request(request: Request<Body>) -> Result<Request<Body>, ProxyError> {
    let (parts, body) = request.into_parts();

    let headers = parts.headers.to_owned();
    let uri = parts.uri.to_owned();
    let method = parts.method.to_owned();
    let version = parts.version;

    let body_bytes = hyper::body::to_bytes(body).await?;

    decode_body(&headers, &body_bytes).await;

    let body = Body::from(body_bytes);
    let request = Request::from_parts(parts, body);
    Ok(request)
}

async fn store_response(response: Response<Body>) -> Result<Response<Body>, ProxyError> {
    let (parts, body) = response.into_parts();

    let headers = parts.headers.to_owned();
    let version = parts.version;

    let body_bytes = hyper::body::to_bytes(body).await?;

    decode_body(&headers, &body_bytes).await;

    let body = Body::from(body_bytes);
    let response = Response::from_parts(parts, body);

    Ok(response)
}

async fn decode_body(
    header: &hyper::HeaderMap,
    body_bytes: &hyper::body::Bytes,
) -> hyper::body::Bytes {
    let mut body_bytes = body_bytes;
    if let Some(content_encoding) = header.get(CONTENT_ENCODING) {
        let methods: Vec<&str> = content_encoding
            .to_str()
            .unwrap()
            .split(',')
            .map(|s| s.trim())
            .collect();

        for method in methods {
            body_bytes = match method {
                "gzip" => {}
                "compress" => {}
                "deflate" => {}
                "identity" => {}
                _ => {}
            }
        }
    }
    body_bytes.to_owned()
}
