#[macro_use]
extern crate tracing;

use axum::{routing::any, Router};
use axum_extra::middleware::option_layer;
use once_cell::sync::Lazy;
use reqwest::{redirect::Policy, Client};
use tokio::signal::{
    ctrl_c,
    unix::{signal, SignalKind},
};
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, validate_request::ValidateRequestHeaderLayer};

mod config;
mod middleware;
mod mikan;
mod trace;
mod util;

pub use config::CONFIG;

pub static CLIENT: Lazy<Client> = Lazy::new(|| init_client());

#[tokio::main]
async fn main() {
    launch_info();
    trace::init();
    info!("listening on http://{}", config::CONFIG.addr);
    let token_layer = match CONFIG.token.clone() {
        Some(t) => Some(ValidateRequestHeaderLayer::custom(middleware::Token::new(
            t,
        ))),
        None => None,
    };
    let token_layer = option_layer(token_layer);
    let layer = ServiceBuilder::new()
        .layer(trace::TraceLayer)
        .layer(CompressionLayer::new())
        .layer(token_layer);

    let app = Router::new()
        .route("/", any(mikan::mikan_proxy))
        .route("/*path", any(mikan::mikan_proxy))
        .layer(layer);

    axum::Server::bind(&config::CONFIG.addr)
        .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

fn launch_info() {
    println!();
    println!(
        "=================== Starting Mikan-Proxy {} ===================",
        env!("CARGO_PKG_VERSION")
    );
    println!();
}

fn init_client() -> Client {
    Client::builder().redirect(Policy::none()).build().unwrap()
}

async fn shutdown_signal() {
    let ctrl_c = async {
        ctrl_c().await.unwrap();
    };

    #[cfg(unix)]
    {
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        let mut sigquit = signal(SignalKind::quit()).unwrap();
        tokio::select! {
            _ = sigint.recv() => {
                info!("SIGINT received; starting forced shutdown");
            },
            _ = sigterm.recv() => {
                info!("SIGTERM received; starting graceful shutdown");
            },
            _ = sigquit.recv() => {
                info!("SIGQUIT received; starting forced shutdown");
            },
            _ = ctrl_c => {
                info!("SIGINT received; starting forced shutdown");
            }
        }
    }

    #[cfg(not(unix))]
    {
        ctrl_c.await;
        println!("SIGINT received; starting forced shutdown");
    }
}
