pub mod ad;
pub mod auth;
pub mod cart;
pub mod dashboard;
pub mod dev;
pub mod kitchen;
pub mod meal;
pub mod media;
pub mod notification;
pub mod order;
pub mod payment;
pub mod storage;
pub mod transaction;
pub mod user;
pub mod wallet;

mod router;
pub use router::get_router;
