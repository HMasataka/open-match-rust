use std::net::AddrParseError;

use thiserror::Error;
use tracing_subscriber::util::TryInitError;

#[derive(Error, Debug)]
pub enum MatchFunctionError {
    #[error("fail to serve. err: {0}")]
    FailToService(tonic::transport::Error),
    #[error("initialize tracing subscriber error. {0}")]
    InitializeTracingSubscriber(TryInitError),
    #[error("addr parse failed. err: {0}")]
    FailedAddrParse(AddrParseError),
}
