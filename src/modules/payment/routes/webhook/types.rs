pub mod request {
    use super::Event;
    use axum::http::header::{HeaderName, HeaderValue};
    use bytes::Bytes;
    use headers::{Error, Header};
    use std::iter;

    pub static X_PAYSTACK_SIGNATURE: HeaderName = HeaderName::from_static("x-paystack-signature");

    #[derive(Clone, Debug)]
    pub struct PaystackSignature(pub String);

    impl Header for PaystackSignature {
        fn name() -> &'static HeaderName {
            &X_PAYSTACK_SIGNATURE
        }

        fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
        where
            Self: Sized,
            I: Iterator<Item = &'i HeaderValue>,
        {
            values
                .next()
                .and_then(|value| value.to_str().ok())
                .map(|value| Self(value.to_string()))
                .ok_or(Error::invalid())
        }

        fn encode<E>(&self, values: &mut E)
        where
            E: Extend<HeaderValue>,
        {
            let bytes = self.0.as_bytes();
            let val =
                HeaderValue::from_bytes(bytes).expect("PaystackSignature is a valid HeaderValue");

            values.extend(iter::once(val))
        }
    }

    pub type Headers = PaystackSignature;

    pub type Json = Event;

    pub struct Payload {
        pub headers: Headers,
        pub body: Bytes,
        pub json: Json,
    }
}

pub mod response {
    use axum::{http::StatusCode, response::IntoResponse};

    pub enum Success {
        Successful,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Successful => StatusCode::OK.into_response(),
            }
        }
    }

    pub enum Error {
        InvalidPayload,
        ServerError,
        OrderNotFound,
        UserNotFound
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::UserNotFound => StatusCode::NOT_FOUND.into_response(),
                Self::OrderNotFound => StatusCode::NOT_FOUND.into_response(),
                Self::InvalidPayload => StatusCode::BAD_REQUEST.into_response(),
                Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_string_from_number;

// #[derive(Deserialize)]
// pub struct CustomerIdentificationFailed {
//     pub email: String,
//     pub reason: String,
// }

// #[derive(Deserialize)]
// pub struct CustomerIdentificationSuccessful {
//     pub email: String,
// }

#[derive(Deserialize)]
pub struct DedicatedAccountAssignmentCustomer {
    #[serde(deserialize_with = "deserialize_string_from_number")]
    pub id: String,
    #[serde(rename = "customer_code")]
    pub code: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct DedicatedAccountAssignmentDedicatedAccountBank {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

#[derive(Deserialize)]
pub struct DedicatedAccountAssignmentDedicatedAccount {
    pub id: i32,
    pub bank: DedicatedAccountAssignmentDedicatedAccountBank,
    pub account_name: String,
    pub account_number: String,
    pub active: bool,
}

#[derive(Serialize, Deserialize)]
pub struct OrderInvoiceMetadata {
    pub order_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct TopupMetadata {
    pub user_id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Metadata {
    Order(OrderInvoiceMetadata),
    Topup(TopupMetadata),
}

#[derive(Deserialize)]
pub struct TransactionSuccessful {
    pub amount: BigDecimal,
    pub metadata: Metadata,
}

#[derive(Deserialize)]
pub struct DedicatedAccountAssignmentSuccessful {
    pub customer: DedicatedAccountAssignmentCustomer,
    pub dedicated_account: DedicatedAccountAssignmentDedicatedAccount,
}

#[derive(Deserialize)]
pub struct DedicatedAccountAssignmentFailed {
    pub customer: DedicatedAccountAssignmentCustomer,
}

#[derive(Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum Event {
    #[serde(rename = "charge.success")]
    TransactionSuccessful(TransactionSuccessful),
    // #[serde(rename = "customeridentification.success")]
    // CustomerIdentificationSuccessful(CustomerIdentificationSuccessful),
    // #[serde(rename = "customeridentification.failed")]
    // CustomerIdentificationFailed(CustomerIdentificationFailed),
    #[serde(rename = "dedicatedaccount.assign.success")]
    DedicatedAccountAssignmentSuccessful(DedicatedAccountAssignmentSuccessful),
    #[serde(rename = "dedicatedaccount.assign.failed")]
    DedicatedAccountAssignmentFailed(DedicatedAccountAssignmentFailed),
}
