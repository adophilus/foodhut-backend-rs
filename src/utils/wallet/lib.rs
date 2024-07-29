use crate::{
    repository::{
        meal::Meal,
        order::{self, Order},
        transaction,
        user::User,
        wallet,
    },
    utils::database::DatabaseConnection,
};

pub enum Error {
    UnexpectedError,
    WalletNotFound,
    InsufficientFunds,
}

pub struct InitializePaymentForMeal {
    payer: User,
    meal: Meal,
}

async fn initialize_payment_for_meal(
    db: DatabaseConnection,
    payload: InitializePaymentForMeal,
) -> Result<(), Error> {
    let wallet = match wallet::find_by_owner_id(db.clone(), payload.payer.id.clone()).await {
        Ok(Some(wallet)) => wallet,
        Ok(None) => return Err(Error::WalletNotFound),
        Err(_) => return Err(Error::UnexpectedError),
    };

    if wallet.balance < payload.meal.price {
        return Err(Error::InsufficientFunds);
    }

    // TODO: switch on the error, because it could just be an insufficient balance
    // TODO: also not sure which of these should come first
    // TODO: these operations need to be processed in a transaction

    if let Err(_) = wallet::update_by_id(
        db.clone(),
        wallet.id.clone(),
        wallet::UpdateWalletPayload {
            operation: wallet::UpdateWalletOperation::Debit,
            amount: payload.meal.price.clone(),
        },
    )
    .await
    {
        return Err(Error::UnexpectedError);
    };

    if let Err(_) = transaction::create(
        db.clone(),
        transaction::CreatePayload {
            amount: payload.meal.price.clone(),
            note: Some(format!("Paid for {}", payload.meal.name.clone())),
            wallet_id: wallet.id.clone(),
        },
    )
    .await
    {
        return Err(Error::UnexpectedError);
    };

    Ok(())
}

struct InitializePaymentForOrder {
    order: Order,
    payer: User,
}

async fn initialize_payment_for_order(
    db: DatabaseConnection,
    payload: InitializePaymentForOrder,
) -> Result<(), Error> {
    let wallet = match wallet::find_by_owner_id(db.clone(), payload.payer.id.clone()).await {
        Ok(Some(wallet)) => wallet,
        Ok(None) => return Err(Error::WalletNotFound),
        Err(_) => return Err(Error::UnexpectedError),
    };

    if wallet.balance < payload.order.total {
        return Err(Error::InsufficientFunds);
    }

    let meals = match order::get_meals_from_order_by_id(db.clone(), payload.order.id).await {
        Ok(meals) => meals,
        Err(_) => return Err(Error::UnexpectedError),
    };

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
