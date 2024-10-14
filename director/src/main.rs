mod openmatch {
    tonic::include_proto!("openmatch");
}
mod err;

use std::time::Duration;

use err::DirectorError;
use openmatch::{
    backend_service_client::BackendServiceClient, function_config, AssignTicketsRequest,
    Assignment, AssignmentGroup, FetchMatchesRequest, FunctionConfig, Match, MatchProfile, Pool,
    TagPresentFilter,
};
use tokio::task::JoinSet;
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use tracing::instrument;
use tracing_error::ErrorLayer;
use tracing_spanned::SpanErr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

const OM_BACKEND_ENDPOINT: &str = "http://open-match-backend.open-match.svc.cluster.local:50505";
const FUNCTION_HOST_NAME: &str = "match-function.open-match.svc.cluster.local";
const FUNCTION_PORT: i32 = 50502;

#[instrument(skip_all, name = "initialize_tracing_subscriber", level = "trace")]
fn initialize_tracing_subscriber() -> Result<(), SpanErr<DirectorError>> {
    tracing_subscriber::Registry::default()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
        )
        .with(ErrorLayer::default())
        .try_init()
        .map_err(DirectorError::InitializeTracingSubscriber)?;

    Ok(())
}

#[tokio::main]
#[instrument(skip_all, name = "main", level = "trace")]
async fn main() -> Result<(), SpanErr<DirectorError>> {
    initialize_tracing_subscriber()?;
    let mut set = JoinSet::new();

    let client = BackendServiceClient::connect(OM_BACKEND_ENDPOINT)
        .await
        .map_err(DirectorError::CreateGrpcClient)?;

    let profiles = generate_profiles();

    for _ in 0..20 {
        let profiles = profiles.clone();
        let client = client.clone();

        for profile in profiles {
            let client = client.clone();

            set.spawn(async move {
                let mut client = client.clone();
                let matches = match fetch(&mut client, profile).await {
                    Ok(m) => m,
                    Err(_) => Vec::<Match>::new(),
                };

                let mut client = client.clone();
                let _ = match assign(&mut client, matches).await {
                    Ok(_) => println!("ok"),
                    Err(e) => println!("{:?}", e),
                };
            });

            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    }

    set.join_all().await;

    Ok(())
}

#[instrument(skip_all, name = "fetch", level = "trace")]
async fn fetch(
    be: &mut BackendServiceClient<Channel>,
    p: MatchProfile,
) -> Result<Vec<Match>, SpanErr<DirectorError>> {
    let req = FetchMatchesRequest {
        config: Some(FunctionConfig {
            host: FUNCTION_HOST_NAME.to_string(),
            port: FUNCTION_PORT,
            r#type: function_config::Type::Grpc.into(),
        }),
        profile: Some(p),
    };

    let mut be = be.clone();

    let mut stream = be
        .fetch_matches(req)
        .await
        .map_err(DirectorError::InvalidTonicStatus)?;

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

#[instrument(skip_all, name = "assign", level = "trace")]
async fn assign(
    be: &mut BackendServiceClient<Channel>,
    matches: Vec<Match>,
) -> Result<(), SpanErr<DirectorError>> {
    for m in matches {
        let mut ticket_ids = Vec::<String>::new();
        for ticket in m.tickets {
            ticket_ids.push(ticket.id);
        }

        let conn = format!("{}.{}.{}.{}:2222", 256, 256, 256, 256);

        let req = AssignTicketsRequest {
            assignments: vec![AssignmentGroup {
                ticket_ids: ticket_ids,
                assignment: Some(Assignment {
                    connection: conn,
                    ..Default::default()
                }),
            }],
        };

        let mut be = be.clone();

        be.assign_tickets(req)
            .await
            .map_err(DirectorError::FailedToAssign)?;
    }

    Ok(())
}

#[instrument(skip_all, name = "generate_profiles", level = "trace")]
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
