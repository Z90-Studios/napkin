use ::config::Config;
use actix_web::{get, middleware::Logger, web, App, HttpServer};
use clap::Parser;
use dotenv::dotenv;
use tokio_postgres::NoTls;

mod config;
mod db;
mod errors;
mod models;
mod services;
use crate::config::NapkinConfig;
use services::projects;
use services::nodes;

struct AppState {
    app_name: String,
}

#[derive(Parser, Debug)]
#[command(author,
    version,
    about,
    long_about = None,
    before_help = "Project:\n███╗   ██╗ █████╗ ██████╗ ██╗  ██╗██╗███╗   ██╗\n████╗  ██║██╔══██╗██╔══██╗██║ ██╔╝██║████╗  ██║\n██╔██╗ ██║███████║██████╔╝█████╔╝ ██║██╔██╗ ██║\n██║╚██╗██║██╔══██║██╔═══╝ ██╔═██╗ ██║██║╚██╗██║\n██║ ╚████║██║  ██║██║     ██║  ██╗██║██║ ╚████║\n╚═╝  ╚═══╝╚═╝  ╚═╝╚═╝     ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝\nA Z90 Studios Project.\n\nCheck out https://z90.studio for documentation and community links.")]
struct Cli {
    #[arg(short, long, default_value = "28527")]
    port: String,
}

#[get("/")]
async fn index(data: web::Data<AppState>) -> String {
    let app_name = &data.app_name;
    format!("{app_name} API")
}

#[rustfmt::skip]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let args = Cli::parse();

    let config_ = Config::builder()
        .add_source(::config::Environment::default())
        .build()
        .unwrap();

    let config: NapkinConfig = config_.try_deserialize().unwrap();

    println!("🚀 {} Started", config.app_name);
    println!("🔧 Listening on {}:{}", config.server_addr, args.port);

    let pool = config.pg.create_pool(None, NoTls).unwrap();

    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    HttpServer::new(move || {
        let logger = Logger::default();

        App::new()
            .wrap(logger)
            .app_data(web::Data::new(AppState {
                app_name: String::from("Project: Napkin"),
            }))
            .app_data(web::Data::new(pool.clone()))
            .service(index)
            .service(
                web::scope("/projects")
                    .service(projects::get_projects)
                    .service(projects::get_project)
                    .service(projects::post_project)
                    .service(projects::delete_project)
            )
            .service(
                web::scope("/nodes")
                    .service(nodes::get_nodes)
                    .service(nodes::get_node)
                    .service(nodes::post_node)
                    .service(nodes::delete_node)
            )
    })
    .bind(format!("{}:{}", config.server_addr, args.port))?
    .run()
    .await
}
