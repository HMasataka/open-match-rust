mod openmatch {
    tonic::include_proto!("openmatch");
}

use openmatch::frontend_service_client::FrontendServiceClient;
use openmatch::CreateTicketRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = FrontendServiceClient::connect("http://[::1]:50504").await?;

    let req = tonic::Request::new(CreateTicketRequest { ticket: None });

    let res = client.create_ticket(req).await?;

    println!("{:?}", res);

    Ok(())
}
