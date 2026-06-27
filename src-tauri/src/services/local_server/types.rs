use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
pub struct OpenTrayRequest {
    pub route: String,
    pub data: serde_json::Value,
}

#[derive(Serialize)]
pub struct PingResponse {
    pub success: bool,
}

#[derive(Deserialize)]
pub struct MessageRequest {
    pub message: String,
}
