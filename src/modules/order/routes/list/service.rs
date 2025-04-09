use super::types::{request, response};
use crate::{
    modules::{order::repository, user},
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let orders = if user::repository::is_admin(&payload.auth.user) {
        repository::find_many_as_admin(
            &ctx.db_conn.pool,
            payload.pagination,
            repository::FindManyAsAdminFilters {
                owner_id: None,
                payment_method: None,
                status: payload.filters.status,
                kitchen_id: payload.filters.kitchen_id,
            },
        )
        .await
    } else {
        if payload.filters.kitchen_id.is_some() {
            repository::find_many_as_kitchen(
                &ctx.db_conn.pool,
                payload.pagination,
                repository::FindManyAsKitchenFilters {
                    // owner_id: Some(auth.user.id.clone()),
                    owner_id: None,
                    payment_method: None,
                    status: payload.filters.status,
                    kitchen_id: payload.filters.kitchen_id,
                },
            )
            .await
        } else {
            repository::find_many_as_user(
                &ctx.db_conn.pool,
                payload.pagination,
                repository::FindManyAsUserFilters {
                    owner_id: Some(payload.auth.user.id),
                    payment_method: None,
                    status: payload.filters.status,
                    kitchen_id: payload.filters.kitchen_id,
                },
            )
            .await
        }
    };

    orders
        .map(response::Success::Orders)
        .map_err(|_| response::Error::FailedToFetchOrders)
}
