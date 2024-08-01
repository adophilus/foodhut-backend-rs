pub mod auth;
pub mod cart;
pub mod kitchen;
pub mod meal;
pub mod media;
pub mod order;
pub mod user;
pub mod webhooks;
// pub mod wallet;

mod router;
pub use router::get_router;
