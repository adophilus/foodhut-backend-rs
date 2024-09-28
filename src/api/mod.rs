pub mod ad;
pub mod auth;
pub mod cart;
pub mod dashboard;
pub mod kitchen;
pub mod meal;
pub mod media;
pub mod notification;
pub mod order;
pub mod user;
pub mod wallet;
pub mod webhooks;

mod router;
pub use router::get_router;
