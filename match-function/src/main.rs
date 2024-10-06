mod openmatch {
    tonic::include_proto!("openmatch");
}

use std::pin::Pin;
use std::time::Duration;

use openmatch::match_function_server::{MatchFunction, MatchFunctionServer};
use openmatch::{RunRequest, RunResponse};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct MMFServer {}

type RunResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<RunResponse, Status>> + Send>>;

#[tonic::async_trait]
impl MatchFunction for MMFServer {
    type RunStream = ResponseStream;

    async fn run(&self, request: Request<RunRequest>) -> RunResult<Self::RunStream> {
        println!("Got a request from {:?}", request.remote_addr());

        let mut replies = Vec::new();
        replies.push(RunResponse { proposal: None });

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = MMFServer {};
    Server::builder()
        .add_service(MatchFunctionServer::new(server))
        .serve("[::1]:50502".parse()?)
        .await?;

    Ok(())
}
