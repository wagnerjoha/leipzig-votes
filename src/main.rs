use actix_files as fs;
use actix_web::web::ServiceConfig;
use shuttle_actix_web::ShuttleActixWeb;

use vis_municipality_votes::app::views_factory;

#[shuttle_runtime::main]
async fn actix_web() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(fs::Files::new("/assets", "./assets").show_files_listing())
           .service(fs::Files::new("/data", "./data").show_files_listing())
           .configure(views_factory);
    };

    Ok(config.into())
}