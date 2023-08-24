use async_trait::async_trait;
use std::net::SocketAddr;

use hyper::{service::service_fn, Body, Client, Request, Response, Server};
use tower::make::Shared;

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
async fn handle(request: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let request = store_request(request).await?;
    let response = get_response(request).await?;
    let response = store_response(response).await?;
    Ok(response)
}

async fn get_response(request: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let client = Client::new();
    client.request(request).await
}

async fn store_request(request: Request<Body>) -> Result<Request<Body>, hyper::Error> {
    let headers = request.headers().to_owned();
    let uri = request.uri().to_owned();
    let method = request.method().to_owned();
    let version = request.version().to_owned();

    let body = hyper::body::to_bytes(request.into_body()).await?;
    let body = String::from_utf8(body.to_vec()).unwrap();

    let request = {
        let mut req = Request::builder().uri(uri).method(method).version(version);
        for (key, value) in headers.iter() {
            req = req.header(key, value);
        }
        req.body(Body::from(body)).unwrap()
    };
    Ok(request)
}

async fn store_response(response: Response<Body>) -> Result<Response<Body>, hyper::Error> {
    let body = hyper::body::to_bytes(response.into_body()).await?;
    let body = String::from_utf8(body.to_vec()).unwrap();
    println!("{}", body);
    let response = Response::new(Body::from(body));
    Ok(response)
}
