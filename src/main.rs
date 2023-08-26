use std::net::SocketAddr;

use hyper::{body::Body, header::CONTENT_ENCODING, service::service_fn, Server};
use thiserror::Error;
use tower::make::Shared;

// ** ####################################################################################################
// ** Suitable Error struct
// ** ####################################################################################################
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("hyper error")]
    HyperError(#[from] hyper::Error),
    #[error("hyper http error")]
    HyperHttpError(#[from] hyper::http::Error),
    #[error("failed to parse hyper's uri as reqwest's url")]
    UriParseError(String),
    #[error("invalid http method is given")]
    InvalidMethodError,
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("send request error")]
    SendRequestError(String),
    #[error("response convert error")]
    ResponseConvertError(String),
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
struct Exchange {
    hyper_request: hyper::Request<hyper::Body>,
    hyper_response: Option<hyper::Response<hyper::Body>>,
    reqw_request: Option<reqwest::RequestBuilder>,
    reqw_response: Option<reqwest::Response>,
    pilot_flag: bool,
}

async fn handle(
    request: hyper::Request<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, ProxyError> {
    let exchange = Exchange::new(request, false);
    exchange.hyper2reqwest();
    exchange.send_request();
}

impl Exchange {
    fn new(hyper_request: hyper::Request<hyper::Body>, pilot_flag: bool) -> Self {
        Exchange {
            hyper_request,
            hyper_response: None,
            reqw_request: None,
            reqw_response: None,
            pilot_flag,
        }
    }

    async fn hyper2reqwest(mut self) -> Result<(), ProxyError> {
        let (parts, body) = self.hyper_request.into_parts();
        let body = hyper::body::to_bytes(body).await?;

        let reqw_body = reqwest::Body::from(body);
        let reqw_headers = reqwest::header::HeaderMap::from(parts.headers);
        let reqw_url = match reqwest::Url::parse(parts.uri.to_string().as_str()) {
            Ok(url) => Ok(url),
            Err(error) => Err(ProxyError::UriParseError(error.to_string())),
        };
        let reqw_url = reqw_url?;
        let reqw_method = match parts.method {
            hyper::http::Method::GET => Ok(reqwest::Method::GET),
            hyper::http::Method::POST => Ok(reqwest::Method::PUT),
            hyper::http::Method::DELETE => Ok(reqwest::Method::DELETE),
            hyper::http::Method::HEAD => Ok(reqwest::Method::HEAD),
            hyper::http::Method::OPTIONS => Ok(reqwest::Method::OPTIONS),
            hyper::http::Method::CONNECT => Ok(reqwest::Method::CONNECT),
            hyper::http::Method::PATCH => Ok(reqwest::Method::PATCH),
            hyper::http::Method::TRACE => Ok(reqwest::Method::TRACE),
            _ => Err(ProxyError::InvalidMethodError),
        };
        let reqw_method = reqw_method?;

        let reqw_client = reqwest::Client::new();
        let reqw_request_builder = reqw_client
            .request(reqw_method, reqw_url)
            .headers(reqw_headers)
            .body(reqw_body);

        self.reqw_request = Some(reqw_request_builder);
        Ok(())
    }

    async fn send_request(mut self) -> Result<(), ProxyError> {
        if let Some(request) = self.reqw_request {
            let response = request.send().await?;
            self.reqw_response = Some(response);
            Ok(())
        } else {
            Err(ProxyError::SendRequestError(format!(
                "reqwest_request property isn't set"
            )))
        }
    }

    async fn reqwest2hyper(mut self) -> Result<(), ProxyError> {
        if let Some(reqw_response) = self.reqw_response {
            let reqw_headers = reqw_response.headers().to_owned();
            let reqw_body = reqw_response.bytes().await?;

            let hyper_body = hyper::Body::from(reqw_body);

            let hyper_response = {
                
                for header in reqw_headers {
                    let mut response = hyper::Response::builder();
                let (name, value) = header;
                if let Some(name) = name {
                    response.header(name,value);
                }
            }};
            Ok(())
        } else {
            Err(ProxyError::ResponseConvertError(format!(
                "reqwest_response property isn't set"
            )))
        }
    }
}

// async fn store_request(request: Request<Body>) -> Result<Request<Body>, ProxyError> {
//     let (parts, body) = request.into_parts();

//     let headers = parts.headers.to_owned();
//     // let uri = parts.uri.to_owned();
//     // let method = parts.method.to_owned();
//     // let version = parts.version;

//     let body_bytes = hyper::body::to_bytes(body).await?;

//     decode_body(&headers, &body_bytes).await;

//     let body = Body::from(body_bytes);
//     let request = Request::from_parts(parts, body);
//     Ok(request)
// }

// async fn store_response(response: Response<Body>) -> Result<Response<Body>, ProxyError> {
//     println!("[response]");
//     let (parts, body) = response.into_parts();

//     let headers = parts.headers.to_owned();
//     // let version = parts.version;

//     let body_bytes = hyper::body::to_bytes(body).await?;
//     // let mut body_string = String::new();
//     // let body_agr = hyper::body::aggregate(body)
//     //     .await
//     //     .unwrap()
//     //     .reader()
//     //     .read_line(&mut body_string);
//     decode_body(&headers, &body_bytes).await;
//     // println!("body string: {}", body_string);
//     // hyper::body::aggregate();

//     let body = Body::from(body_bytes);
//     let response = Response::from_parts(parts, body);

//     Ok(response)
// }

// async fn decode_body(
//     header: &hyper::HeaderMap,
//     body_bytes: &hyper::body::Bytes,
// ) -> hyper::body::Bytes {
//     let mut body_bytes = body_bytes;
//     if let Some(content_encoding) = header.get(CONTENT_ENCODING) {
//         let methods: Vec<&str> = content_encoding
//             .to_str()
//             .unwrap()
//             .split(',')
//             .map(|s| s.trim())
//             .collect();

//         for method in methods {
//             body_bytes = match method {
//                 "gzip" => body_bytes,
//                 "compress" => body_bytes,
//                 "deflate" => body_bytes,
//                 "identity" => body_bytes,
//                 _ => body_bytes,
//             }
//         }
//     }
//     body_bytes.to_owned()
// }
