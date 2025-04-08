use super::types::{request, response};
use crate::{
    modules::{
        cart::{self, repository::CreateCartPayload},
        meal::repository,
    },
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let meal = repository::find_by_id(&ctx.db_conn.pool, payload.id)
        .await
        .map_err(|_| response::Error::FailedToFetchMeal)?
        .ok_or(response::Error::MealNotFound)?;

    match payload.auth {
        Some(auth) => {
            let cart = match cart::repository::find_active_cart_by_owner_id(
                &ctx.db_conn.pool,
                auth.user.id.clone(),
            )
            .await
            .map_err(|_| response::Error::FailedToFetchMeal)?
            {
                Some(cart) => cart,
                _ => cart::repository::create(
                    &ctx.db_conn.pool,
                    CreateCartPayload {
                        owner_id: auth.user.id.clone(),
                    },
                )
                .await
                .map_err(|_| response::Error::FailedToFetchMeal)?,
            };

            Ok(response::Success::Meal(meal.with_cart_status(&cart)))
        }
        _ => Ok(response::Success::Meal(meal.into())),
    }
}
