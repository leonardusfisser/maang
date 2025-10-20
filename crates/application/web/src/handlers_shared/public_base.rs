use actix_web::{HttpResponse, Responder};
use askama::Template;
use tracing::{info, error};
use time::OffsetDateTime;

use core_config::load_site_info;

#[derive(Debug)]
pub struct BaseData<'a> {
    pub title: &'a str,
    pub site_name: &'a str,
    pub site_title: &'a str,
    pub tagline: &'a str,
    pub shared_nav_main: &'a str,
    pub css_style: &'a str,
    pub css_default: &'a str,
    pub current_year: &'a str,
    pub powered_by: &'a str,
}

#[derive(Debug, Template)]
#[template(path = "public_home.html")]
pub struct PublicHomeTemplate<'a> {
    pub base: BaseData<'a>,
}

pub async fn home() -> impl Responder {

    let site = load_site_info();
    info!(target: "http.home", site = %site.site_name, "Rendering public_home");

    let year = OffsetDateTime::now_utc().year().to_string();

    let base_data = BaseData {
        title: &site.site_title,
        site_name: &site.site_name,
        site_title: &site.site_title,
        tagline: &site.tagline,
        shared_nav_main: "",
        css_style: "/static/css/style.css",
        css_default: "/static/css/default.css",
        current_year: &year,
        powered_by: "maangframe.com",
    };

    let tpl = PublicHomeTemplate {
        base: base_data,
    };

    match tpl.render() {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            error!(target: "http.home", ?e, "template render error");
            HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body("Template rendering error")
        }
    }
}