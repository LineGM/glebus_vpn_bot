use once_cell::sync::Lazy;
use remnawave::RemnawaveApiClient;
use std::sync::Arc;

static CLIENT: Lazy<Arc<RemnawaveApiClient>> = Lazy::new(|| {
    Arc::new(
        RemnawaveApiClient::new(
            dotenv::var("PANEL_BASE_URL").expect("PANEL_BASE_URL must be set"),
            Some(dotenv::var("REMNAWAVE_API_TOKEN").expect("REMNAWAVE_API_TOKEN must be set")),
        )
        .expect("Failed to create RemnawaveApiClient"),
    )
});

/// Returns a shared reference to the RemnawaveApiClient.
pub fn get_client() -> Arc<RemnawaveApiClient> {
    CLIENT.clone()
}
