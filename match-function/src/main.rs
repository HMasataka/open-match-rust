mod openmatch {
    tonic::include_proto!("openmatch");
}
mod err;
mod server;

use std::net::SocketAddr;

use err::MatchFunctionError;
use server::make_server;
use tonic::transport::Server;
use tracing::instrument;
use tracing_error::ErrorLayer;
use tracing_spanned::SpanErr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[instrument(skip_all, name = "initialize_tracing_subscriber", level = "trace")]
fn initialize_tracing_subscriber() -> Result<(), SpanErr<MatchFunctionError>> {
    tracing_subscriber::Registry::default()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
        )
        .with(ErrorLayer::default())
        .try_init()
        .map_err(MatchFunctionError::InitializeTracingSubscriber)?;

    Ok(())
}

#[tokio::main]
#[instrument(skip_all, name = "main", level = "trace")]
async fn main() -> Result<(), SpanErr<MatchFunctionError>> {
    initialize_tracing_subscriber()?;

    let addr = "[::0]:50502"
        .parse::<SocketAddr>()
        .map_err(MatchFunctionError::FailedAddrParse)?;

    println!("{}", addr);

    Server::builder()
        .add_service(make_server())
        .serve(addr)
        .await
        .map_err(MatchFunctionError::FailToService)?;

    Ok(())
}
