#[cfg(not(test))] use {
    rust_server_v2::login_service::{wait_for_shudown},
    rust_server_v2::request_handling::handle_request,
    std::convert::Infallible,
    std::net::SocketAddr,
    hyper::Server,
    hyper::service::{make_service_fn, service_fn},
    anyhow::{Result},
};

#[cfg(not(test))]
#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let make_svc = make_service_fn( |_connection| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });
    let server = Server::bind(&addr).serve(make_svc);
    let graceful_server = server.with_graceful_shutdown(wait_for_shudown());
    graceful_server.await?;
    Ok(())
}