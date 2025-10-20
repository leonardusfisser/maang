use crate::handlers_labeled;
use actix_web::web;

pub fn routing_labeled(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/admin/labeled").route(web::get().to(handlers_labeled::admin_labeled::get)));
    cfg.service(web::resource("/labeled").route(web::get().to(handlers_labeled::public_labeled::get)));
}


