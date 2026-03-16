use std::sync::LazyLock;

use reqwest::ClientBuilder;
use reqwest_middleware::{ClientBuilder as ClientWithMiddlewareBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};

const UPLOAD_RETRY_COUNT: u32 = 3;
const OIDC_RETRY_COUNT: u32 = 10;
const USER_AGENT: &str = "codspeed-runner";

pub static REQUEST_CLIENT: LazyLock<ClientWithMiddleware> = LazyLock::new(|| {
    ClientWithMiddlewareBuilder::new(ClientBuilder::new().user_agent(USER_AGENT).build().unwrap())
        .with(RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff::builder().build_with_max_retries(UPLOAD_RETRY_COUNT),
        ))
        .build()
});

/// Client without retry middleware for streaming uploads (can't be cloned)
pub static STREAMING_CLIENT: LazyLock<reqwest::Client> =
    LazyLock::new(|| ClientBuilder::new().user_agent(USER_AGENT).build().unwrap());

/// Client with retry middleware for OIDC token requests
pub static OIDC_CLIENT: LazyLock<ClientWithMiddleware> = LazyLock::new(|| {
    ClientWithMiddlewareBuilder::new(ClientBuilder::new().user_agent(USER_AGENT).build().unwrap())
        .with(RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff::builder().build_with_max_retries(OIDC_RETRY_COUNT),
        ))
        .build()
});
