mod openmatch {
    tonic::include_proto!("openmatch");
}

use rand::seq::SliceRandom;
use tokio::task::JoinSet;
use tonic::transport::Channel;

use openmatch::frontend_service_client::FrontendServiceClient;
use openmatch::{CreateTicketRequest, DeleteTicketRequest, GetTicketRequest, SearchFields, Ticket};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FrontendServiceClient::connect("http://[::1]:50504").await?;
    let mut set = JoinSet::new();

    for _ in 0..20 {
        let mut client = client.clone();

        let ticket = create_new_ticket();
        let req = tonic::Request::new(CreateTicketRequest {
            ticket: Some(ticket.clone()),
        });

        let res = client.create_ticket(req).await?;

        println!("{:?}", res);

        set.spawn(async move {
            delete_on_assign(&mut client, res.get_ref().clone()).await;
        });
    }

    set.join_all().await;

    Ok(())
}

async fn delete_on_assign(client: &mut FrontendServiceClient<Channel>, ticket: Ticket) {
    loop {
        let got = client
            .get_ticket(GetTicketRequest {
                ticket_id: ticket.clone().id,
            })
            .await
            .unwrap();

        let res = got.get_ref().clone().assignment;

        if res.is_some() {
            println!("{:?}", res);
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    let d = client
        .delete_ticket(DeleteTicketRequest {
            ticket_id: ticket.clone().id,
        })
        .await
        .unwrap();

    println!("{:?}", d);
}

fn create_new_ticket() -> Ticket {
    Ticket {
        search_fields: Some(SearchFields {
            tags: game_mode(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn game_mode() -> Vec<String> {
    let v = vec!["mode.demo", "mode.ctf", "mode.battleroyale"];

    let sample = v
        .choose_multiple(&mut rand::thread_rng(), 1)
        .map(|&s| s.into())
        .collect();

    return sample;
}
