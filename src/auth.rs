use anyhow::{Context, Result};
use reqwest::Client;
use tracing::info;
use wp_mini::WattpadClient;
use crate::error::AppError;

pub async fn login(
    wp_client: &WattpadClient,
    username: &str,
    password: &str
) -> Result<(), AppError> {
    info!(username, "Attempting to login via core::auth");
    wp_client
        .authenticate(username, password)
        .await
        .context("Wattpad login request failed")
        .map_err(|_e| AppError::AuthenticationFailed)?;

    Ok(())
}

pub async fn logout(wp_client: &WattpadClient) -> Result<()> {
    info!("Attempting to logout via core::auth");
    wp_client
        .deauthenticate()
        .await
        .context("Wattpad logout request failed")
        .map_err(|_| AppError::LogoutFailed)?;
    Ok(())
}
