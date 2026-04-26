use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use payrail::{
    LipilaConfig, PayRail, PayRailClient, PaymentError, PaymentProvider, WebhookRequest,
};
use secrecy::SecretString;

#[derive(Debug, Clone)]
struct AppState {
    client: PayRailClient,
}

async fn lipila_webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    match state
        .client
        .parse_webhook(
            PaymentProvider::Lipila,
            WebhookRequest::new(body.as_ref(), headers),
        )
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(PaymentError::WebhookVerificationFailed) => StatusCode::UNAUTHORIZED,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

fn webhook_router(client: PayRailClient) -> Router {
    Router::new()
        .route("/webhooks/lipila", post(lipila_webhook_handler))
        .with_state(AppState { client })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PayRail::builder()
        .lipila(
            LipilaConfig::sandbox(SecretString::from(std::env::var("LIPILA_API_KEY")?))?
                .webhook_secret(Some(SecretString::from(std::env::var(
                    "LIPILA_WEBHOOK_SECRET",
                )?))),
        )?
        .build()?;
    let app = webhook_router(client);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
