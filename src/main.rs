use actix_web::{ get, web, App, HttpServer, middleware::Logger };

mod services;
mod models;
mod errors;
use services::projects;

struct AppState {
    app_name: String,
}

#[get("/")]
async fn index(data: web::Data<AppState>) -> String {
    let app_name = &data.app_name;
    format!("{app_name} API")
}

#[rustfmt::skip]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    HttpServer::new(|| {
        let logger = Logger::default();

        App::new()
            .wrap(logger)
            .app_data(web::Data::new(AppState {
                app_name: String::from("Project: Napkin"),
            }))
            .service(index)
            .service(
                web::scope("/projects")
                    .service(projects::get_projects)
                    .service(projects::get_project)
                    .service(projects::post_project)
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
