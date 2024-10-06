mod openmatch {
    tonic::include_proto!("openmatch");
}

use openmatch::{match_function_client::MatchFunctionClient, MatchProfile, RunRequest};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MatchFunctionClient::connect("http://[::1]:50502").await?;

    let req = tonic::Request::new(RunRequest {
        profile: Some(MatchProfile {
            name: "".to_string(),
            ..Default::default()
        }),
    });

    let mut stream = client.run(req).await?.into_inner();

    while let Some(item) = stream.next().await {
        println!("{:?}", item.unwrap())
    }

    Ok(())
}
