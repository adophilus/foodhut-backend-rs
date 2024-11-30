pub mod ad;
pub mod auth;
pub mod cart;
pub mod dashboard;
pub mod kitchen;
pub mod meal;
pub mod media;
pub mod notification;
pub mod order;
pub mod transaction;
pub mod user;
pub mod wallet;
pub mod payment;

mod router;
pub use router::get_router;
