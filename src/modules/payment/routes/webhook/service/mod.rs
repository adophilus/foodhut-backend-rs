mod handler;

use super::types::{request, response, Event};
use crate::types::Context;

use bytes::Bytes;
use hmac::{Hmac, Mac};
use sha2::Sha512;
use std::sync::Arc;

fn verify_header(
    ctx: Arc<Context>,
    header: request::PaystackSignature,
    body: Bytes,
) -> Result<(), response::Error> {
    let mut mac =
        Hmac::<Sha512>::new_from_slice(ctx.payment.secret_key.as_bytes()).map_err(|err| {
            tracing::error!("Failed to generate mac: {:?}", err);
            response::Error::InvalidPayload
        })?;

    mac.update(body.as_ref());

    mac.verify_slice(hex::decode(header.0).expect("Invalid hex header").as_ref())
        .map_err(|err| {
            tracing::error!("Failed to verify header: {:?}", err);
            response::Error::InvalidPayload
        })
        .map(|_| ())
}

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    verify_header(ctx.clone(), payload.headers.clone(), payload.body.clone())?;

    let body_string = String::from_utf8(payload.body.into()).map_err(|err| {
        tracing::error!("Failed to convert body bytes to string: {:?}", err);
        response::Error::ServerError
    })?;

    tracing::debug!("Trying to parse body: {}", body_string);

    match payload.json {
        Event::TransactionSuccessful(payload) => {
            handler::transaction_successful(ctx, payload).await
        }

        // Event::CustomerIdentificationFailed(payload) => {
        //     handler::customer_identification_failed(ctx.clone(), payload)
        //         .await
        //         .into_response()
        // }
        // Event::CustomerIdentificationSuccessful(payload) => {
        //     handler::customer_identification_successful(ctx.clone(), payload)
        //         .await
        //         .into_response()
        // }
        Event::DedicatedAccountAssignmentSuccessful(payload) => {
            handler::dedicated_account_assignment_successful(ctx.clone(), payload).await
        }
        Event::DedicatedAccountAssignmentFailed(payload) => {
            handler::dedicated_account_assignment_failed(ctx.clone(), payload).await
        }
    }
}
