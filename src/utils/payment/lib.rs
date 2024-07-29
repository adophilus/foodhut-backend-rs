use crate::repository::{order::Order, user::User};
use crate::types::DatabaseConnection;
use crate::utils::wallet;

pub enum Error {
    UnexpectedError,
}

pub enum PaymentMethod {
    Wallet,
    Online,
}

pub struct InitializePaymentForOrder {
    payer: User,
    order: Order,
}

#[derive(Serialize)]
pub struct PaymentDetails {
    url: Option<String>,
}

async fn initialize_payment_for_order(
    db: DatabaseConnection,
    method: PaymentMethod,
    payload: InitializePaymentForOrder,
) -> Result<PaymentDetails, Error> {
    match method {
        Wallet => match wallet::initialize_payment_for_order(
            db,
            wallet::InitializePaymentForOrder {
                order: payload.order,
                payer: payload.payer,
            },
        )
        .await
        {
            Ok(_) => Ok(PaymentDetails { url: None }),
            Err(_) => Err(Error::UnexpectedError),
        },
        Online => unimplemented!(),
    }
}
