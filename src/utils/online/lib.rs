use bigdecimal::BigDecimal;

use crate::{
    repository::{
        meal::Meal,
        order::{self, Order},
        transaction,
        user::User,
    },
    types::Context,
    utils::database::DatabaseConnection,
};

pub enum Error {
    UnexpectedError,
}

async fn create_payment_link(ctx: Context, amount: BigDecimal) -> Result<String, Error> {

}

struct InitializePaymentForOrder {
    order: Order,
    payer: User,
}

async fn initialize_payment_for_order(
    ctx: Context,
    payload: InitializePaymentForOrder,
) -> Result<(), Error> {
    let meals = match order::get_meals_from_order_by_id(db.clone(), payload.order.id).await {
        Ok(meals) => meals,
        Err(_) => return Err(Error::UnexpectedError),
    };

    let payment_link = create_payment_link(ctx, payload.order.total);

    for meal in meals {
        match initialize_payment_for_meal(
            db.clone(),
            InitializePaymentForMeal {
                meal,
                payer: payload.payer.clone(),
            },
        )
        .await
        {
            Ok(_) => (),
            Err(_) => return Err(Error::UnexpectedError),
        }
    }

    Ok(())
}
