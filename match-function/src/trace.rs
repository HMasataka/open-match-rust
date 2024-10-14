use crate::err::MatchFunctionError;

use tracing::instrument;
use tracing_error::ErrorLayer;
use tracing_spanned::SpanErr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[instrument(skip_all, name = "initialize_tracing_subscriber", level = "trace")]
pub fn initialize_tracing_subscriber() -> Result<(), SpanErr<MatchFunctionError<'static>>> {
    tracing_subscriber::Registry::default()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
        )
        .with(ErrorLayer::default())
        .try_init()
        .map_err(MatchFunctionError::InitializeTracingSubscriber)?;

    Ok(())
}
