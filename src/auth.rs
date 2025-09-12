use anyhow::{Context, Result};
use reqwest::Client;
use tracing::info;
use wp_mini::WattpadClient;
use crate::error::AppError;

pub async fn login(client: &Client, username: &str, password: &str) -> Result<(), AppError> {
    info!(username, "Attempting to login via core::auth");
    let wp_client = WattpadClient::builder()
        .reqwest_client(client.clone())
        .build();

    wp_client
        .authenticate(username, password)
        .await
        .map_err(|_e| AppError::AuthenticationFailed)?;
    Ok(())
}

pub async fn logout(client: &Client) -> Result<()> {
    info!("Attempting to logout via core::auth");
    let wp_client = WattpadClient::builder()
        .reqwest_client(client.clone())
        .build();

    wp_client
        .deauthenticate()
        .await
        .context("Wattpad logout request failed")
        .map_err(|_| AppError::LogoutFailed)?;
    Ok(())
}
