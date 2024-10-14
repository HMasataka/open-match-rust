use std::pin::Pin;
use std::time::Duration;

use crate::openmatch::match_function_server::{MatchFunction, MatchFunctionServer};
use crate::openmatch::query_service_client::QueryServiceClient;
use crate::openmatch::{Match, RunRequest, RunResponse};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};
use tracing::instrument;

use crate::err::MatchFunctionError;
use crate::query::Query;

pub struct MMFServer {
    query: Query,
}

type RunResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<RunResponse, Status>> + Send>>;

impl MMFServer {
    pub fn new() -> Self {
        let query = Query::new();

        MMFServer { query }
    }
}

const OM_QUERY_ENDPOINT: &str = "http://open-match-query.open-match.svc.cluster.local:50503";

#[tonic::async_trait]
impl MatchFunction for MMFServer {
    type RunStream = ResponseStream;

    #[instrument(skip_all, name = "run", level = "trace")]
    async fn run(&self, request: Request<RunRequest>) -> RunResult<Self::RunStream> {
        println!("Got a request from {:?}", request.remote_addr());

        let mut client = QueryServiceClient::connect(OM_QUERY_ENDPOINT)
            .await
            .map_err(|_| Status::unknown("create client error"))?;

        let tickets = self
            .query
            .query_pool(&mut client)
            .await
            .map_err(|_| Status::unknown("get tickets error"))?;
        println!("{:?}", tickets);

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

#[instrument(skip_all, name = "make_server", level = "trace")]
pub async fn make_server() -> Result<MatchFunctionServer<MMFServer>, MatchFunctionError> {
    let server = MMFServer::new();

    Ok(MatchFunctionServer::new(server))
}
