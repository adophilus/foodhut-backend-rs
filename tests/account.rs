use reqwest::Client;
use serde_json::json;
use serde::Deserialize;

#[derive(Deserialize)]
struct SignUpResponse {
    message: String
}

#[test]
async fn create_account() {
    let client = reqwest::Client::new();

    let response = client.post("http://localhost:8000/api/auth/sign-up/strategy/credentials").body(json!({
        "email": "",
        "phone_number": "",
        "first_name": "",
        "last_name": ""
    }).to_string()).send().await.unwrap().json::<SignUpResponse>().await.unwrap();
}
