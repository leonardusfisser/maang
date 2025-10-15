#![forbid(unsafe_code)]
#![deny(warnings)]

pub mod domain_error;
pub mod app_error;
pub mod infra_error;
pub mod http_error;

pub use {
    domain_error::*,
    app_error::*,
    infra_error::*,
    http_error::*
};