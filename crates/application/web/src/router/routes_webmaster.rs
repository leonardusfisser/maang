use actix_web::web;
use crate::handlers_webmaster;


pub fn routing_webmaster(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/metrics").route(web::get().to(handlers_webmaster::admin_webmaster::get)));
    cfg.service(web::resource("/metrics").route(web::get().to(handlers_webmaster::public_webmaster::get)));
}

