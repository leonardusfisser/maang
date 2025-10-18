pub mod routes_labeled;
pub mod routes_shared;
pub mod routes_webmaster;

pub use {
  routes_labeled::routing_labeled,
  routes_shared::routing_shared,
  routes_webmaster::routing_webmaster,
};