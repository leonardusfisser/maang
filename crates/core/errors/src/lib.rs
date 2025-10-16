#![forbid(unsafe_code)]
#![deny(warnings)]

pub mod domain_error;
pub mod app_error;
pub mod infrastructure_error;
pub mod http_error;

pub use {
    domain_error::*,
    app_error::*,
    infrastructure_error::*,
    http_error::*
};