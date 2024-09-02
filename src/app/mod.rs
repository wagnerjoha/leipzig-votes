use actix_web::web::ServiceConfig;
use actix_web::{web, HttpResponse};

use crate::app::data::{data_items, read_file};

pub mod data;

pub fn views_factory(app: &mut ServiceConfig) {
    app_views_factory(app);
}

pub fn app_views_factory(app: &mut ServiceConfig) {
    app.route("/", web::get().to(main_page));
    app.route("/Ergebnisse", web::get().to(data_items));
    app.route(
        "/maps/2024_Stadtratswahl",
        web::get().to(|| map_view("Leipzig_Stadtratswahl_2024")),
    );
    app.route(
        "/maps/2024_Europawahl",
        web::get().to(|| map_view("Leipzig_Europawahl_2024")),
    );
    app.route(
        "/maps/2024_Landtagswahl",
        web::get().to(|| map_view("Leipzig_Landtagswahl_2024")),
    );
}

pub async fn map_view(data_source: &str) -> HttpResponse {
    println!("{}", data_source);
    let mut html_map = read_file("./templates/map.html");
    let javascript_data = read_file("./src/javascript/map.js");

    let filename = format!("const DATA_SOURCE = '{}';", data_source);
    html_map = html_map
        .replace("{{JAVASCRIPT}}", &(filename + &javascript_data))
        .replace("{{DATA_TITLE}}", &data_source.replace("_", " "));

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_map)
}

async fn main_page() -> HttpResponse {
    let html_main = read_file("./templates/main.html");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_main)
}
