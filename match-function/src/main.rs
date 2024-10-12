mod openmatch {
    tonic::include_proto!("openmatch");
}
mod err;

use std::pin::Pin;
use std::time::Duration;

use err::MatchFunctionError;
use openmatch::match_function_server::{MatchFunction, MatchFunctionServer};
use openmatch::{Match, RunRequest, RunResponse};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tracing::instrument;
use tracing_error::ErrorLayer;
use tracing_spanned::SpanErr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[derive(Default)]
pub struct MMFServer {}

type RunResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<RunResponse, Status>> + Send>>;

#[tonic::async_trait]
impl MatchFunction for MMFServer {
    type RunStream = ResponseStream;

    #[instrument(skip_all, name = "run", level = "trace")]
    async fn run(&self, request: Request<RunRequest>) -> RunResult<Self::RunStream> {
        println!("Got a request from {:?}", request.remote_addr());

        let replies = make_matches();

        let mut stream = Box::pin(tokio_stream::iter(replies).throttle(Duration::from_millis(200)));

        let (tx, rx) = mpsc::channel(128);
        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                match tx.send(Result::<_, Status>::Ok(item)).await {
                    Ok(_) => {} // item (server response) was queued to be send to client
                    Err(_item) => {
                        // output_stream was build from rx and both are dropped
                        break;
                    }
                }
            }

            println!("\tclient disconnected");
        });

        let output_stream = ReceiverStream::new(rx);

        Ok(Response::new(Box::pin(output_stream) as Self::RunStream))
    }
}

#[instrument(skip_all, name = "make_matches", level = "trace")]
fn make_matches() -> Vec<RunResponse> {
    let mut replies = Vec::new();

    for i in 1..4 {
        replies.push(RunResponse {
            proposal: Some(Match {
                match_id: format!("{}", i),
                ..Default::default()
            }),
        });
    }

    return replies;
}

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

    let addr = "[::1]:50502";
    let server = MMFServer {};

    println!("{}", addr);

    Server::builder()
        .add_service(MatchFunctionServer::new(server))
        .serve(addr.parse().map_err(MatchFunctionError::FailedAddrParse)?)
        .await
        .map_err(MatchFunctionError::FailToService)?;

    Ok(())
}
