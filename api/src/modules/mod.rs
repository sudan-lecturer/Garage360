pub mod auth;
pub mod customers;
pub mod vehicles;
pub mod control;
pub mod tenant;

pub mod routes {
    pub use crate::modules::control::routes as control;
    pub use crate::modules::tenant::routes as tenant;
}