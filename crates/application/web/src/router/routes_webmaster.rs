use actix_web::{HttpResponse, Responder, web};

use crate::handlers_webmaster::{
    self,
    admin_webmaster,
    public_webmaster,
};


pub fn routing(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/metrics").route(web::get().to(handlers_webmaster::admin_webmaster::get)));
    cfg.service(web::resource("/metrics").route(web::get().to(handlers_webmaster::public_webmaster::get)));
}

