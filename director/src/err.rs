use thiserror::Error;
use tracing_subscriber::util::TryInitError;

#[derive(Error, Debug)]
pub enum DirectorError {
    #[error("initialize tracing subscriber error. {0}")]
    InitializeTracingSubscriber(TryInitError),
    #[error("create grpc client error. err: {0}")]
    CreateGrpcClient(tonic::transport::Error),
    #[error("invalid tonic status. status: {0}")]
    InvalidTonicStatus(tonic::Status),
    #[error("assign error. status: {0}")]
    FailedToAssign(tonic::Status),
}
