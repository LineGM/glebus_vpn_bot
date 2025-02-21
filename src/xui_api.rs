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
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            session_cookie: None,
        }
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), MyError> {
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
            Err(MyError::from(
                format!("Login failed with status: {}", response.status()).as_str(),
            ))
        }
    }

    fn with_cookie(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref cookie) = self.session_cookie {
            req.header(COOKIE, cookie)
        } else {
            req
        }
    }

    pub async fn get_inbounds(&self) -> Result<Value, MyError> {
        let url = format!("{}/panel/api/inbounds/list", self.base_url);
        let response = self.with_cookie(self.client.get(&url)).send().await?;
        let json: Value = response.json().await?;
        Ok(json)
    }

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
}
