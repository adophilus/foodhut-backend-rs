use reqwest::Client;

#[test]
async fn create_account() {
    let client = reqwest::Client::new();

    let response = client.post("http://localhost:8000/api/auth/sign-up/strategy/credentials").body(json!({
        email: "",
        phone_number: "",
        first_name: "",
        last_name: ""
    }).send().await.json::<SignUpResponse>().await;
}
