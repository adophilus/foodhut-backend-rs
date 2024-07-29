pub mod auth;
pub mod cart;
pub mod kitchen;
pub mod meal;
pub mod media;
pub mod order;
pub mod user;
// pub mod wallet;

mod lib;
pub use lib::get_router;
