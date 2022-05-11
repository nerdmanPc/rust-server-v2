use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Method, StatusCode};
use hyper::body::Bytes;
use futures::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let make_svc = make_service_fn( |_connection| async {
        Ok::<_, Infallible>(service_fn(main_service))
    });
    let server = Server::bind(&addr).serve(make_svc);
    let graceful_server = server.with_graceful_shutdown(wait_for_shudown());
    graceful_server.await?;
    Ok(())
}

async fn main_service(request: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    
    let mut response = Response::new(Body::empty());
    match (request.method(), request.uri().path()) {
        (&Method::GET, "/login") => {
            let html_contents = include_str!("../pages/login.html");
            *response.body_mut() = Body::from(html_contents);
        },
        (&Method::GET, "/default_avatar.png") => {
            let avatar_data: &[u8] = include_bytes!("../pages/default_avatar.png");
            //let avatar_data = Bytes::from(avatar_data);
            *response.body_mut() = Body::from(avatar_data);
        }
        (&Method::POST, "/echo") => {
            *response.body_mut() = request.into_body()
        },
        (&Method::POST, "/echo/uppercase") => {
            let uppercase_fn = |byte: &u8| { byte.to_ascii_uppercase() };
            let body_build_fn = move |chunk: Bytes| { chunk.iter().map(uppercase_fn).collect::<Vec<u8>>() };              //KKKKKK
            let mapping = request.into_body().map_ok(body_build_fn);
            *response.body_mut() = Body::wrap_stream(mapping);
        },
        (&Method::POST, "/echo/reverse") => {
            let full_body = hyper::body::to_bytes(request.into_body()).await?;
            let reversed_body = full_body.iter().rev().cloned().collect::<Vec<u8>>();
            *response.body_mut() = reversed_body.into();
        },
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }
    Ok(response)
}

async fn wait_for_shudown() {
    tokio::signal::ctrl_c().await.expect("Failed to initialize Ctrl+C signal handler");
}