use std::{
    net::AddrParseError,
    sync::{MutexGuard, PoisonError},
};

use thiserror::Error;
use tonic::{transport::Channel, Status};
use tracing_subscriber::util::TryInitError;

use crate::openmatch::query_service_client::QueryServiceClient;

#[derive(Error, Debug)]
pub enum MatchFunctionError<'a> {
    #[error("fail to serve. err: {0}")]
    FailToService(tonic::transport::Error),
    #[error("initialize tracing subscriber error. {0}")]
    InitializeTracingSubscriber(TryInitError),
    #[error("addr parse failed. err: {0}")]
    FailedAddrParse(AddrParseError),
    #[error("initialize client error. err: {0}")]
    ClientConnect(tonic::transport::Error),
    #[error("receive query ticket error. err: {0}")]
    ReceiveQueryTickets(Status),
    #[error("mutex lock error. err: {0}")]
    Lock(PoisonError<MutexGuard<'a, QueryServiceClient<Channel>>>),
}
