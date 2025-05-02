use super::types::{request, response};
use crate::{
    modules::{order::repository, user},
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    if user::repository::is_admin(&payload.auth.user) {
        let order = repository::find_full_order_by_id(&ctx.db_conn.pool, payload.id)
            .await
            .map_err(|_| response::Error::FailedToFetchOrder)?
            .ok_or(response::Error::OrderNotFound)?;

        user::repository::find_by_id(&ctx.db_conn.pool, order.owner_id.clone())
            .await
            .map_err(|_| response::Error::FailedToFetchOrder)?
            .ok_or(response::Error::FailedToFetchOrder)
            .map(|owner| response::Success::OrderWithOwner(order.with_owner(owner)))
    } else {
        // TODO: please add an 'as_kitchen' field here to make it more secure
        // repository::find_full_order_by_id_and_owner_id(
        //     &ctx.db_conn.pool,
        //     payload.id,
        //     payload.auth.user.id,
        // )
        // .await
        // .map_err(|_| response::Error::FailedToFetchOrder)?
        // .ok_or(response::Error::OrderNotFound)
        // .map(response::Success::Order)
        repository::find_full_order_by_id(&ctx.db_conn.pool, payload.id)
            .await
            .map_err(|_| response::Error::FailedToFetchOrder)?
            .ok_or(response::Error::OrderNotFound)
            .map(response::Success::Order)
    }
}
