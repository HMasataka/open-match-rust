mod openmatch {
    tonic::include_proto!("openmatch");
}
mod err;
mod server;
mod trace;

use std::net::SocketAddr;

use err::MatchFunctionError;
use server::make_server;
use tonic::transport::Server;
use trace::initialize_tracing_subscriber;
use tracing::instrument;
use tracing_spanned::SpanErr;

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
