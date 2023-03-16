use std::time::Duration;

use reqwest::ClientBuilder;

/// Checks crates.io for updates to pest_language_server.
/// Returns the latest version if not already installed, otherwise [None].
pub async fn check_for_updates() -> Option<String> {
    let client = ClientBuilder::new()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .timeout(Duration::from_secs(2))
        .build();

    if let Ok(client) = client {
        let response = client
            .get("https://crates.io/api/v1/crates/pest_language_server")
            .send()
            .await;

        if let Ok(response) = response {
            return response
                .json::<serde_json::Value>()
                .await
                .map_or(None, |json| {
                    let version = json["crate"]["max_version"].as_str()?;

                    if version != env!("CARGO_PKG_VERSION") {
                        Some(version.to_string())
                    } else {
                        None
                    }
                });
        }
    }

    None
}
