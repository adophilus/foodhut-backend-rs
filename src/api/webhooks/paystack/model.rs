use bigdecimal::BigDecimal;
use serde::Deserialize;

use crate::utils;

#[derive(Deserialize)]
pub struct CustomerIdentificationFailed {
    pub email: String,
    pub reason: String,
}

#[derive(Deserialize)]
pub struct CustomerIdentificationSuccessful {
    pub email: String,
}

#[derive(Deserialize)]
pub struct DedicatedAccountAssignmentCustomer {
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
    TransactionSuccessful {
        amount: BigDecimal,
        metadata: utils::online::Metadata,
    },
    #[serde(rename = "customeridentification.success")]
    CustomerIdentificationSuccessful(CustomerIdentificationSuccessful),
    #[serde(rename = "customeridentification.failed")]
    CustomerIdentificationFailed(CustomerIdentificationFailed),
    #[serde(rename = "dedicatedaccount.assign.success")]
    DedicatedAccountAssignmentSuccessful(DedicatedAccountAssignmentSuccessful),
    #[serde(rename = "dedicatedaccount.assign.failed")]
    DedicatedAccountAssignmentFailed(DedicatedAccountAssignmentFailed),
}
