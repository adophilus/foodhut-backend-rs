use axum::Router;

use crate::{
    api::{auth, kitchen, user},
    types::Context,
};
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .nest("/auth", auth::get_router())
        .nest("/users", user::get_router())
        .nest("/kitchens", kitchen::get_router())
    // .layer(axum::middleware::from_fn(auth::middleware::auth))
}
