mod openmatch {
    tonic::include_proto!("openmatch");
}

use crate::err::MatchFunctionError;
use openmatch::{
    query_service_client::QueryServiceClient, Pool, QueryTicketsRequest, TagPresentFilter, Ticket,
};
use tokio_stream::StreamExt;
use tonic::transport::Channel;

pub struct Query {
    client: QueryServiceClient<Channel>,
}

const OM_QUERY_ENDPOINT: &str = "http://open-match-query.open-match.svc.cluster.local:50503";

impl Query {
    pub async fn new() -> Result<Self, MatchFunctionError> {
        let client = QueryServiceClient::connect(OM_QUERY_ENDPOINT)
            .await
            .map_err(MatchFunctionError::ClientConnect)?;

        Ok(Self { client })
    }

    pub async fn query_pool(&mut self) -> Result<Vec<Ticket>, MatchFunctionError> {
        let mode = "mode.demo";

        let req = tonic::Request::new(QueryTicketsRequest {
            pool: Some(Pool {
                name: format!("pool_mode_{}", mode),
                tag_present_filters: vec![TagPresentFilter {
                    tag: mode.to_string(),
                }],
                ..Default::default()
            }),
        });

        let mut queries = self
            .client
            .query_tickets(req)
            .await
            .map_err(MatchFunctionError::ReceiveQueryTickets)?;

        let stream = queries.get_mut();

        let mut tickets = Vec::new();

        while let Some(item) = stream.next().await {
            match item {
                Ok(m) => {
                    for ticket in m.tickets {
                        tickets.push(ticket);
                    }
                }
                Err(_item) => {
                    break;
                }
            }
        }

        Ok(tickets)
    }
}
