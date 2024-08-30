use actix_files as fs;
use actix_service::Service;
use actix_web::{App, HttpServer};

use vis_municipality_votes::app::views_factory;

// mod app;
// mod harvester;
// mod structs;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Let's start our Server
    HttpServer::new(|| {
        let app = App::new()
            .wrap_fn(|req, srv| {
                let future = srv.call(req);
                async {
                    let result = future.await?;
                    Ok(result)
                }
            })
            .service(fs::Files::new("/assets", "./assets").show_files_listing())
            .service(fs::Files::new("/data", "./data").show_files_listing())
            .configure(views_factory);
        return app;
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
