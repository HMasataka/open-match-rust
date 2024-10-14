use std::pin::Pin;
use std::time::{Duration, SystemTime};

use crate::openmatch::match_function_server::{MatchFunction, MatchFunctionServer};
use crate::openmatch::query_service_client::QueryServiceClient;
use crate::openmatch::{Match, MatchProfile, RunRequest, RunResponse, Ticket};
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

        let replies = make_matches(request.get_ref().profile.clone(), tickets);

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

const TICKET_PER_MATCH: usize = 4;

#[instrument(skip_all, name = "make_matches", level = "trace")]
fn make_matches(p: Option<MatchProfile>, tickets: Vec<Ticket>) -> Vec<RunResponse> {
    let mut matches = Vec::new();

    if p.is_none() {
        return matches;
    }

    let mut slice = &tickets[0..];
    let mut count = 0;

    loop {
        if slice.len() < TICKET_PER_MATCH {
            break;
        }

        let match_ticket = slice[0..TICKET_PER_MATCH].to_vec();
        slice = &slice[TICKET_PER_MATCH..];

        matches.push(RunResponse {
            proposal: Some(Match {
                match_id: format!(
                    "profile-{}-{:?}-{}",
                    p.as_ref().unwrap().name,
                    SystemTime::now(),
                    count
                ),
                match_profile: p.as_ref().unwrap().name.clone(),
                match_function: "basic".to_string(),
                tickets: match_ticket,
                ..Default::default()
            }),
        });

        count += 1;
    }

    return matches;
}

#[instrument(skip_all, name = "make_server", level = "trace")]
pub async fn make_server() -> Result<MatchFunctionServer<MMFServer>, MatchFunctionError> {
    let server = MMFServer::new();

    Ok(MatchFunctionServer::new(server))
}
