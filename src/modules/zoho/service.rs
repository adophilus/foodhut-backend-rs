use crate::{modules::user::repository::User, types::Context};
use axum::http::{HeaderMap, Method, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::Arc;

pub enum Error {
    UnexpectedError,
    RequestNotSent,
    InvalidHttpResponseStatusCode,
    FailedToDecodeResponse,
}

pub async fn generate_token_link(ctx: Arc<Context>) -> Result<String, Error> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|err| {
            tracing::error!("Failed to create token generator http client: {:?}", err);
            Error::UnexpectedError
        })?;

    let response = client
        .get(format!("{}/oauth/v2/auth", ctx.zoho.accounts_api_endpoint))
        .query(&[
            ("response_type", "code"),
            ("client_id", &ctx.zoho.client_id),
            ("scope", "ZohoCampaigns.contact.CREATE"),
            ("redirect_uri", &ctx.zoho.redirect_url),
            ("access_type", "offline"),
        ])
        .send()
        .await
        .map_err(|_| Error::UnexpectedError)?;

    let oauth_url = response
        .headers()
        .get("location")
        .ok_or(Error::UnexpectedError)?
        .to_str()
        .map_err(|err| {
            tracing::error!("Failed to convert oauth url to string slice: {:?}", err);
            Error::UnexpectedError
        })?;

    Ok(String::from(oauth_url))
}

pub struct ExchangePayload {
    pub grant_code: String,
    pub account_server_url: String,
}

#[derive(Deserialize, Serialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn exchange_grant_code_for_tokens(
    ctx: Arc<Context>,
    payload: ExchangePayload,
) -> Result<Tokens, Error> {
    send_zoho_request::<Tokens>(
        ctx.clone(),
        SendZohoRequestPayload {
            target: ZohoRequestTarget::Accounts,
            query: Some(&[
                ("client_id", ctx.zoho.client_id.as_str()),
                ("client_secret", ctx.zoho.client_secret.as_str()),
                ("grant_type", "authorization_code"),
                ("code", payload.grant_code.as_str()),
                ("redirect_uri", ctx.zoho.redirect_url.as_str()),
            ]),
            body: None,
            route: String::from("/oauth/v2/token"),
            method: Method::POST,
            expected_status_code: StatusCode::OK,
        },
    )
    .await
}

#[derive(Deserialize)]
pub struct RefreshAccessTokenResponse {
    pub access_token: String,
}

pub async fn refresh_access_token(ctx: Arc<Context>) -> Result<(), Error> {
    let response = send_zoho_request::<RefreshAccessTokenResponse>(
        ctx.clone(),
        SendZohoRequestPayload {
            target: ZohoRequestTarget::Accounts,
            query: Some(&[
                ("client_id", ctx.zoho.client_id.as_str()),
                ("client_secret", ctx.zoho.client_secret.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", ctx.zoho.refresh_token.as_str()),
            ]),
            body: None,
            route: String::from("/oauth/v2/token"),
            method: Method::POST,
            expected_status_code: StatusCode::OK,
        },
    )
    .await?;

    let mut access_token = ctx.zoho.access_token.lock().await;
    *access_token = response.access_token;

    Ok(())
}

#[derive(Deserialize)]
struct UserRegistrationApiResponse {
    status: String,
    message: Option<String>,
}

pub async fn register_user(ctx: Arc<Context>, user: User) -> Result<(), Error> {
    let response = send_zoho_request::<UserRegistrationApiResponse>(
        ctx.clone(),
        SendZohoRequestPayload {
            target: ZohoRequestTarget::Campaigns,
            query: Some(&[
                ("listkey", ctx.zoho.campaigns_list_key.as_str()),
                ("resfmt", "JSON"),
                ("emailids", user.email.as_str()),
            ]),
            body: None,
            route: String::from("/api/v1.1/addlistsubscribersinbulk"),
            method: Method::POST,
            expected_status_code: StatusCode::OK,
        },
    )
    .await?;

    if response.status != "success" {
        tracing::error!(
            "Got an error from the server: {}",
            response.message.unwrap()
        );
        return Err(Error::UnexpectedError);
    }

    Ok(())
}

enum ZohoRequestTarget {
    Campaigns,
    Accounts,
}

struct SendZohoRequestPayload<'a> {
    pub route: String,
    pub body: Option<String>,
    pub expected_status_code: StatusCode,
    pub method: Method,
    pub query: Option<&'a [(&'a str, &'a str)]>,
    pub target: ZohoRequestTarget,
}

async fn send_zoho_request<'a, R: DeserializeOwned>(
    ctx: Arc<Context>,
    payload: SendZohoRequestPayload<'a>,
) -> Result<R, Error> {
    let mut headers = HeaderMap::new();
    let auth_header = format!("Bearer {}", *ctx.zoho.access_token.lock().await);
    headers.insert(
        "Authorization",
        auth_header
            .clone()
            .try_into()
            .expect("Invalid auth header value"),
    );
    let target = match payload.target {
        ZohoRequestTarget::Accounts => &ctx.zoho.accounts_api_endpoint,
        ZohoRequestTarget::Campaigns => &ctx.zoho.campaigns_api_endpoint,
    };
    let url = format!("{}{}", target, payload.route);
    let client = reqwest::Client::new();
    let mut req = match payload.method {
        Method::GET => client.get(url),
        _ => client.post(url),
    };

    req = req.headers(headers);

    match payload.query {
        Some(query) => req = req.query(query),
        _ => (),
    };

    match payload.body {
        Some(body) => {
            req = req.body(body);
        }
        None => (),
    };

    let res = req.send().await.map_err(|err| {
        tracing::error!("Failed to send Zoho request: {}", err);
        Error::RequestNotSent
    })?;

    let http_response_status_code = res.status();

    if http_response_status_code != payload.expected_status_code {
        tracing::error!(
            "Got unexpected http response status: {}",
            http_response_status_code
        );
        Err(Error::InvalidHttpResponseStatusCode)?
    }

    let data = res.text().await.map_err(|err| {
        tracing::error!("Failed to get text of failed Zoho request: {}", err);
        Error::InvalidHttpResponseStatusCode
    })?;

    tracing::trace!("Response received from Zoho server: {}", data);

    let zoho_response = serde_json::de::from_str::<R>(&data).map_err(|err| {
        tracing::error!("Failed to decode Zoho response: {}", err);
        Error::FailedToDecodeResponse
    })?;

    Ok(zoho_response)
}
