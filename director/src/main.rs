mod openmatch {
    tonic::include_proto!("openmatch");
}

use openmatch::{
    backend_service_client::BackendServiceClient, function_config, FetchMatchesRequest,
    FunctionConfig, Match, MatchProfile, Pool, TagPresentFilter,
};
use tokio_stream::StreamExt;
use tonic::transport::Channel;

const OM_BACKEND_ENDPOINT: &str = "http://open-match-backend.open-match.svc.cluster.local:50505";
const FUNCTION_HOST_NAME: &str = "http://match-function.open-match.svc.cluster.local";
const FUNCTION_PORT: i32 = 50502;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BackendServiceClient::connect(OM_BACKEND_ENDPOINT).await?;

    let profiles = generate_profiles();

    for profile in profiles {
        let mut client = client.clone();
        let matches = fetch(&mut client, profile).await.unwrap();

        let mut client = client.clone();
        assign(&mut client, matches).await.unwrap();
    }

    Ok(())
}

async fn fetch(
    be: &mut BackendServiceClient<Channel>,
    p: MatchProfile,
) -> Result<Vec<Match>, Box<dyn std::error::Error>> {
    let req = FetchMatchesRequest {
        config: Some(FunctionConfig {
            host: FUNCTION_HOST_NAME.to_string(),
            port: FUNCTION_PORT,
            r#type: function_config::Type::Grpc.into(),
        }),
        profile: Some(p),
    };

    let mut be = be.clone();
    let mut stream = be.fetch_matches(req).await.unwrap();
    let stream = stream.get_mut();

    let mut matches = Vec::new();

    while let Some(item) = stream.next().await {
        match item {
            Ok(m) => {
                matches.push(m.r#match.unwrap());
            }
            Err(_item) => {
                break;
            }
        }
    }

    Ok(matches)
}

async fn assign(
    _be: &mut BackendServiceClient<Channel>,
    _matches: Vec<Match>,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn generate_profiles() -> Vec<MatchProfile> {
    let mut profiles = Vec::new();
    let modes = vec!["mode.demo", "mode.ctf", "mode.battleroyale"];

    for mode in modes {
        profiles.push(MatchProfile {
            name: "mode_based_profile".to_string(),
            pools: vec![Pool {
                name: format!("pool_mode_{}", mode),
                tag_present_filters: vec![TagPresentFilter {
                    tag: mode.to_string(),
                }],
                ..Default::default()
            }],
            ..Default::default()
        })
    }

    return profiles;
}
