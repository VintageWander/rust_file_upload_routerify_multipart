#![allow(unused_variables, dead_code)]

use bytes::BufMut;
use std::net::SocketAddr;
use uuid::Uuid;

use hyper::{Body, Request, Response, Server, StatusCode};
use routerify::{Error, Router, RouterService};
use routerify_multipart::RequestMultipartExt;
use serde::{Deserialize, Serialize};
use serde_json::json;

async fn hello(req: Request<Body>) -> Result<Response<Body>, Error> {
    Ok(Response::new(Body::from("Hello world")))
}

#[derive(Serialize, Deserialize)]
struct UploadResponse {
    name: String,
    filename: String,
    status: String,
}
impl UploadResponse {
    fn new(name: String, filename: String, status: String) -> Self {
        UploadResponse {
            name,
            filename,
            status,
        }
    }
}

async fn file_upload(req: Request<Body>) -> Result<Response<Body>, Error> {
    let mut multipart = match req.into_multipart() {
        Ok(m) => m,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!("Bad request: {}", e)))
                .unwrap())
        }
    };

    let mut responses = vec![];
    let mut bytes = vec![];

    while let Some(mut field) = multipart.next_field().await.map_err(Error::wrap)? {
        // println!("{:#?}", field.headers());
        // println!("{:#?}", field.content_type());
        // println!("{:#?}", field.file_name());
        // println!("{:#?}", field.index());
        // println!("{:#?}", field.name());

        responses.push(json!(&UploadResponse::new(
            field.name().unwrap().to_owned(),
            field.file_name().unwrap().to_owned(),
            "File uploaded successfully".to_string()
        )));

        while let Some(chunk) = field.chunk().await.map_err(Error::wrap)? {
            // Do something with field chunk.
            bytes.put(chunk);
        }
        let file_name = format!("./files/{}_{}", Uuid::new_v4(), field.file_name().unwrap());
        tokio::fs::write(&file_name, bytes.clone())
            .await
            .map_err(|e| {
                eprintln!("Error writting file: {}", e);
                Error::wrap(e)
            })?;
    }
    Ok(Response::new(Body::from(
        serde_json::to_string_pretty(&responses).unwrap(),
    )))
}

fn router() -> Router<Body, Error> {
    Router::builder()
        .get("/", hello)
        .post("/upload", file_upload)
        .build()
        .expect("Server cannot be started")
}

#[tokio::main]
async fn main() {
    let router = router();
    let service = RouterService::new(router).expect("Cannot start service");
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let server = Server::bind(&addr).serve(service);
    println!("App is running on {}", addr);
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
