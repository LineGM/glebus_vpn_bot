use reqwest::header::{CONTENT_TYPE, COOKIE};
use reqwest::Client;
use serde_json::Value;
use std::error::Error;

pub struct ThreeXUiClient {
    pub client: Client,
    pub base_url: String,
    // Stores the session cookie returned on login.
    pub session_cookie: Option<String>,
}

impl ThreeXUiClient {
    /// Create a new 3x‑ui API client with the given base URL (e.g., "http://localhost:2053").
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            session_cookie: None,
        }
    }

    /// Log in by POSTing username and password to the /login endpoint.
    /// On success, extracts and stores the session cookie.
    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), Box<dyn Error>> {
        let url = format!("{}/login", self.base_url);
        let params = [("username", username), ("password", password)];
        let response = self.client.post(&url).form(&params).send().await?;
        if response.status().is_success() {
            if let Some(cookie) = response.headers().get("set-cookie") {
                let cookie_str = cookie.to_str()?.to_string();
                self.session_cookie = Some(cookie_str);
            }
            Ok(())
        } else {
            Err(format!("{}", response.status()).into())
        }
    }

    /// Helper: attach the session cookie to a request if available.
    fn with_cookie(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref cookie) = self.session_cookie {
            req.header(COOKIE, cookie)
        } else {
            req
        }
    }

    /// Retrieve all inbounds by sending a GET request to "/panel/api/inbounds/list".
    pub async fn get_inbounds(&self) -> Result<Value, Box<dyn Error>> {
        let url = format!("{}/panel/api/inbounds/list", self.base_url);
        let response = self.with_cookie(self.client.get(&url)).send().await?;
        let json: Value = response.json().await?;
        Ok(json)
    }

    /// Retrieve details for a specific inbound identified by its ID.
    /// Sends a GET request to "/panel/api/inbounds/get/{id}".
    pub async fn get_inbound(&self, inbound_id: u32) -> Result<Value, Box<dyn Error>> {
        let url = format!("{}/panel/api/inbounds/get/{}", self.base_url, inbound_id);
        let response = self.with_cookie(self.client.get(&url)).send().await?;
        let json: Value = response.json().await?;
        Ok(json)
    }

    /// Add a new inbound by sending a JSON configuration to "/panel/api/inbounds/add".
    pub async fn add_inbound(&self, inbound_config: &Value) -> Result<Value, Box<dyn Error>> {
        let url = format!("{}/panel/api/inbounds/add", self.base_url);
        let response = self
            .with_cookie(
                self.client
                    .post(&url)
                    .header(CONTENT_TYPE, "application/json")
                    .json(inbound_config),
            )
            .send()
            .await?;
        let json: Value = response.json().await?;
        Ok(json)
    }

    /// Add a new client to an existing inbound.
    ///
    /// This sends a POST request to "/panel/api/inbounds/addClient" with a payload that
    /// includes the inbound ID and the client configuration (serialized as a JSON string).
    pub async fn add_client(
        &self,
        inbound_id: u32,
        client_config: &Value,
    ) -> Result<Value, Box<dyn Error>> {
        let url = format!("{}/panel/api/inbounds/addClient", self.base_url);
        let payload = serde_json::json!({
            "id": inbound_id,
            "settings": serde_json::to_string(client_config)?
        });
        let response = self
            .with_cookie(
                self.client
                    .post(&url)
                    .header(CONTENT_TYPE, "application/json")
                    .json(&payload),
            )
            .send()
            .await?;
        let json: Value = response.json().await?;
        Ok(json)
    }
}
