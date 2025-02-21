use crate::error::MyError;
use reqwest::header::{CONTENT_TYPE, COOKIE};
use reqwest::Client;
use serde_json::Value;

pub struct ThreeXUiClient {
    pub client: Client,
    pub base_url: String,
    pub session_cookie: Option<String>,
}

impl ThreeXUiClient {
    /// Creates a new instance of `ThreeXUiClient`.
    ///
    /// Initializes a `ThreeXUiClient` with the provided base URL, a new HTTP client,
    /// and no session cookie.
    ///
    /// # Arguments
    ///
    /// * `base_url` - A string slice representing the base URL for the client.
    ///
    /// # Returns
    ///
    /// An instance of `ThreeXUiClient`.
    pub fn new(base_url: &str) -> Self {
        // Create a new instance of the client with the given base URL.
        // A new HTTP client is created and the session cookie is initially set to None.
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            session_cookie: None,
        }
    }

    /// Logs in to the panel using the provided username and password.
    ///
    /// This function sends a POST request to "/login" with the given username and password
    /// in form data format. If the response is successful, it extracts the session cookie
    /// from the "set-cookie" header and stores it in the client's state. Otherwise, it
    /// returns an error with the status code.
    ///
    /// # Arguments
    ///
    /// * `username` - A string slice representing the username for login.
    /// * `password` - A string slice representing the password for login.
    ///
    /// # Returns
    ///
    /// A `Result` indicating the success of the login. If the login is successful, it returns
    /// `Ok(())`. If the login fails, it returns an error with the status code.
    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), MyError> {
        let url = format!("{}/login", self.base_url);
        let params = [("username", username), ("password", password)];
        let response = self.client.post(&url).form(&params).send().await?;

        // If the response is successful, extract the session cookie from the
        // "set-cookie" header and store it in the client's state.
        if response.status().is_success() {
            if let Some(cookie) = response.headers().get("set-cookie") {
                let cookie_str = cookie.to_str()?.to_string();
                self.session_cookie = Some(cookie_str);
            }
            Ok(())
        } else {
            // If the response is not successful, return an error with the status code.
            Err(MyError::from(
                format!("Login failed with status: {}", response.status()).as_str(),
            ))
        }
    }

    /// Attaches the session cookie to the request if available.
    ///
    /// This function checks if a session cookie is present and appends it to
    /// the provided request as a header. If no session cookie is present, the
    /// request is returned unmodified.
    ///
    /// # Arguments
    ///
    /// * `req` - A `reqwest::RequestBuilder` representing the request to which
    /// the session cookie should be attached.
    ///
    /// # Returns
    ///
    /// A `reqwest::RequestBuilder` with the session cookie attached if available.
    fn with_cookie(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref cookie) = self.session_cookie {
            req.header(COOKIE, cookie)
        } else {
            req
        }
    }

    /// Retrieves a list of all inbound configurations from the panel.
    ///
    /// This function sends a GET request to "/panel/api/inbounds/list"
    /// and returns the JSON response containing a list of inbound configurations.
    ///
    /// # Returns
    ///
    /// A `Result` wrapping a `Value` containing the JSON response. If an error occurs
    /// during the request or response parsing, a `MyError` is returned.
    pub async fn get_inbounds(&self) -> Result<Value, MyError> {
        let url = format!("{}/panel/api/inbounds/list", self.base_url);
        let response = self.with_cookie(self.client.get(&url)).send().await?;
        let json: Value = response.json().await?;
        Ok(json)
    }

    /// Retrieves the inbound configuration for a specified inbound ID.
    ///
    /// This function sends a GET request to "/panel/api/inbounds/get/<inbound_id>"
    /// and returns the JSON response containing the inbound configuration.
    ///
    /// # Arguments
    ///
    /// * `inbound_id` - A 32-bit unsigned integer representing the ID of the inbound.
    ///
    /// # Returns
    ///
    /// A `Result` wrapping a `Value` containing the JSON response. If an error occurs
    /// during the request or response parsing, a `MyError` is returned.
    pub async fn get_inbound(&self, inbound_id: u32) -> Result<Value, MyError> {
        let url = format!("{}/panel/api/inbounds/get/{}", self.base_url, inbound_id);
        let response = self.with_cookie(self.client.get(&url)).send().await?;
        let json: Value = response.json().await?;
        Ok(json)
    }

    /// Adds a new inbound configuration to the panel.
    ///
    /// This function sends a POST request to "/panel/api/inbounds/add" with the given
    /// inbound configuration in JSON format.
    ///
    /// # Arguments
    ///
    /// * `inbound_config` - A reference to a `Value` representing the configuration of the inbound to be added.
    ///
    /// # Returns
    ///
    /// A `Result` wrapping a `Value` containing the JSON response from the panel. If an error occurs
    /// during the request or response parsing, a `MyError` is returned.
    pub async fn add_inbound(&self, inbound_config: &Value) -> Result<Value, MyError> {
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

    /// Adds a new client to the panel.
    ///
    /// This function sends a POST request to "/panel/api/inbounds/addClient" with the given
    /// inbound ID and client configuration in JSON format.
    ///
    /// # Arguments
    ///
    /// * `inbound_id` - A 32-bit unsigned integer representing the ID of the inbound to
    /// which the client should be added.
    /// * `client_config` - A reference to a `Value` representing the configuration of the client
    /// to be added.
    ///
    /// # Returns
    ///
    /// A `Result` wrapping a `Value` containing the JSON response from the panel. If an error occurs
    /// during the request or response parsing, a `MyError` is returned.
    pub async fn add_client(
        &self,
        inbound_id: u32,
        client_config: &Value,
    ) -> Result<Value, MyError> {
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

    pub async fn update_client(&self, uuid: &str, client_config: &Value) -> Result<Value, MyError> {
        let url = format!("{}/panel/api/inbounds/updateClient/{}", self.base_url, uuid);
        let response = self
            .with_cookie(
                self.client
                    .post(&url)
                    .header(CONTENT_TYPE, "application/json")
                    .json(client_config),
            )
            .send()
            .await?;
        let json: Value = response.json().await?;
        Ok(json)
    }

    pub async fn delete_client(&self, inbound_id: u32, uuid: &str) -> Result<Value, MyError> {
        let url = format!(
            "{}/panel/api/inbounds/{}/delClient/{}",
            self.base_url, inbound_id, uuid
        );
        let response = self
            .with_cookie(
                self.client
                    .post(&url)
                    .header(CONTENT_TYPE, "application/json"),
            )
            .send()
            .await?;
        let json: Value = response.json().await?;
        Ok(json)
    }

    pub async fn has_existing_client(&self, tg_id: i64) -> Result<bool, MyError> {
        let inbound = self.get_inbound(1).await?;

        if let Some(settings_str) = inbound
            .get("obj")
            .and_then(|obj| obj.get("settings").and_then(|s| s.as_str()))
        {
            if let Ok(settings_json) = serde_json::from_str::<Value>(settings_str) {
                if let Some(clients) = settings_json.get("clients").and_then(|c| c.as_array()) {
                    return Ok(clients.iter().any(|client| {
                        client.get("tgId").and_then(|id| id.as_i64()) == Some(tg_id)
                    }));
                }
            }
        }

        Ok(false)
    }

    pub async fn get_client_connections(&self, tg_id: i64) -> Result<String, MyError> {
        let inbound = self.get_inbound(1).await?;

        if let Some(settings_str) = inbound
            .get("obj")
            .and_then(|obj| obj.get("settings").and_then(|s| s.as_str()))
        {
            if let Ok(settings_json) = serde_json::from_str::<Value>(settings_str) {
                if let Some(clients) = settings_json.get("clients").and_then(|c| c.as_array()) {
                    let user_clients: Vec<&serde_json::Value> = clients
                        .iter()
                        .filter(|client| {
                            client.get("tgId").and_then(|id| id.as_i64()) == Some(tg_id)
                        })
                        .collect();

                    if !user_clients.is_empty() {
                        return Ok(serde_json::to_string_pretty(&user_clients)?);
                    }
                }
            }
        }

        Ok("{}".to_string())
    }
}
