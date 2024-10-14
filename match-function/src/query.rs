use crate::err::MatchFunctionError;
use crate::openmatch::{
    query_service_client::QueryServiceClient, Pool, QueryTicketsRequest, TagPresentFilter, Ticket,
};
use tokio_stream::StreamExt;
use tonic::transport::Channel;

pub struct Query {}

impl Query {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn query_pool(
        &self,
        client: &mut QueryServiceClient<Channel>,
    ) -> Result<Vec<Ticket>, MatchFunctionError> {
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

        let mut queries = client
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
