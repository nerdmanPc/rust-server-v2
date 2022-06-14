
#[cfg(not(test))] use {
    crate::login_table::LoginTable,
    crate::login_service::{login, signup},
    hyper::{Body, Request, Response, Method, StatusCode},
    anyhow::Result,
};


#[cfg(not(test))]
pub async fn handle_request(request: Request<Body>) -> Result<Response<Body>> {

    match request.method() {
        &Method::GET => {
            return handle_get(request).await;
        },
        &Method::POST => {
            return handle_post(request).await;
        }
        _ => {
            return handle_not_found().await;
        }
    }
}

#[cfg(not(test))]
async fn handle_get(request: Request<Body>) -> Result<Response<Body>> {

    let mut response = Response::new(Body::empty());
    match request.uri().path() {
        "/login" => {
            let html_contents = include_str!("../pages/login.html");
            *response.body_mut() = Body::from(html_contents);
        },
        "/signup" => {
            let html_contents = include_str!("../pages/signup.html");
            *response.body_mut() = Body::from(html_contents);
        },
        "/default_avatar.png" => {
            let avatar_data: &[u8] = include_bytes!("../pages/default_avatar.png");
            *response.body_mut() = Body::from(avatar_data);
        }
        _ => {
            return handle_not_found().await;
        }
    }
    Ok(response)
}

#[cfg(not(test))]
async fn handle_post(request: Request<Body>) -> Result<Response<Body>> {
    let mut response = Response::new(Body::empty());
    match request.uri().path() {
        "/login/try" => {
            let login_table = LoginTable::new().await.unwrap();
            let login_params: &[u8] = &(*hyper::body::to_bytes(request.into_body()).await?);
            let login_params: &str = std::str::from_utf8(login_params)?;
            let login_result = login(&login_table, login_params).await;
            if let Err(e) = login_result {
                *response.status_mut() = StatusCode::BAD_REQUEST;
                *response.body_mut() = Body::from(format!("Error cause: {}", e));
                return Ok(response);
            }
            *response.body_mut() = Body::from("Login successful!");
        },
        "/signup/try" => {
            let mut login_table = LoginTable::new().await.unwrap();
            let signup_params: &[u8] = &(*hyper::body::to_bytes(request.into_body()).await?);
            let signup_params: &str = std::str::from_utf8(signup_params)?;
            let signup_result = signup(&mut login_table, signup_params).await;
            if let Err(e) = signup_result {
                *response.status_mut() = StatusCode::BAD_REQUEST;
                *response.body_mut() = Body::from(format!("Error cause: {}", e));
                return Ok(response);
            }
            *response.body_mut() = Body::from("Signup successful!");
        },
        _ => {
            return handle_not_found().await;
        },
    }
    Ok(response)
}

#[cfg(not(test))]
async fn handle_not_found() -> Result<Response<Body>> {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::NOT_FOUND;
    Ok(response)
}