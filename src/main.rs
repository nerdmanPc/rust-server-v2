use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Method, StatusCode};
use anyhow::{Result};
use hyper::body::Bytes;
use futures::TryStreamExt;
use std::process::exit;

mod lib; use lib::*;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let make_svc = make_service_fn( |_connection| async {
        Ok::<_, Infallible>(service_fn(main_service))
    });
    let server = Server::bind(&addr).serve(make_svc);
    let graceful_server = server.with_graceful_shutdown(wait_for_shudown());
    graceful_server.await?;
    Ok(())
}

async fn main_service(request: Request<Body>) -> Result<Response<Body>> {
    
    let mut response = Response::new(Body::empty());
    let mut login_table = LoginTable::new();//LoginTable::load_or_create("database/file.json").unwrap();

    match (request.method(), request.uri().path()) {
        (&Method::GET, "/login") => {
            let html_contents = include_str!("../pages/login.html");
            *response.body_mut() = Body::from(html_contents);
        },
        (&Method::POST, "/login/try") => {
            let login_params: &[u8] = &(*hyper::body::to_bytes(request.into_body()).await?);
            let login_params: &str = std::str::from_utf8(login_params)?;
            let LoginForm { user_name, password, .. } = parse_login_params(login_params)?;
            let login_result = login_table.login(user_name.as_str(), password.as_str());
            if let Err(e) = login_result {
                *response.status_mut() = StatusCode::BAD_REQUEST;
                *response.body_mut() = Body::from(format!("Error cause: {}", e));
                return Ok(response);
            }
            *response.body_mut() = Body::from("Login successful!");
        },
        (&Method::GET, "/signup") => {
            let html_contents = include_str!("../pages/signup.html");
            *response.body_mut() = Body::from(html_contents);
        },
        (&Method::POST, "/signup/try") => {
            let signup_params: &[u8] = &(*hyper::body::to_bytes(request.into_body()).await?);
            let signup_params: &str = std::str::from_utf8(signup_params)?;
            let SignupForm { user_name, password, psw_repeat, .. } = parse_signup_params(signup_params)?;
            let signup_result = login_table.signup(user_name.as_str(), password.as_str(), psw_repeat.as_str());
            if let Err(e) = signup_result {
                *response.status_mut() = StatusCode::BAD_REQUEST;
                *response.body_mut() = Body::from(format!("Error cause: {}", e));
            }
            *response.body_mut() = Body::from("Signup successful!");
        },
        (&Method::GET, "/default_avatar.png") => {
            let avatar_data: &[u8] = include_bytes!("../pages/default_avatar.png");
            *response.body_mut() = Body::from(avatar_data);
        },
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }
    Ok(response)
}
