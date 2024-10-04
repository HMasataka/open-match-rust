mod openmatch {
    tonic::include_proto!("openmatch");
}

use rand::seq::SliceRandom;
use std::collections::HashMap;

use openmatch::frontend_service_client::FrontendServiceClient;
use openmatch::{CreateTicketRequest, SearchFields, Ticket};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = FrontendServiceClient::connect("http://[::1]:50504").await?;

    let ticket = create_new_ticket().unwrap();
    let req = tonic::Request::new(CreateTicketRequest {
        ticket: Some(ticket),
    });

    let res = client.create_ticket(req).await?;

    println!("{:?}", res);

    Ok(())
}

fn create_new_ticket() -> Result<Ticket, Box<dyn std::error::Error>> {
    Ok(Ticket {
        id: "1".to_string(),
        assignment: None,
        search_fields: Some(SearchFields {
            double_args: HashMap::new(),
            string_args: HashMap::new(),
            tags: game_mode(),
        }),
        persistent_field: HashMap::new(),
        extensions: HashMap::new(),
        create_time: None,
    })
}

fn game_mode() -> Vec<String> {
    let v = vec!["mode.demo", "mode.ctf", "mode.battleroyale"];

    let sample = v
        .choose_multiple(&mut rand::thread_rng(), 1)
        .map(|&s| s.into())
        .collect();

    return sample;
}
