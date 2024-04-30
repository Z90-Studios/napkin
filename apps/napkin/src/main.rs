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
use services::{projects, nodes, edges, node_metadata, edge_metadata};

pub struct AppState {
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
                web::scope("/project")
                    .service(projects::get_projects)
                    .service(projects::get_project)
                    .service(projects::post_project)
                    .service(projects::update_project)
                    .service(projects::delete_project)
            )
            .service(
                web::scope("/node")
                    .service(web::scope("/metadata")
                        .service(node_metadata::get_node_metadata)
                        .service(node_metadata::get_node_metadata_singleton)
                        .service(node_metadata::get_node_metadata_singleton_key)
                        .service(node_metadata::update_node_metadata)
                        .service(node_metadata::post_node_metadata)
                        .service(node_metadata::delete_node_metadata)
                    )
                    .service(nodes::get_nodes)
                    .service(nodes::get_node)
                    .service(nodes::post_node)
                    .service(nodes::update_node)
                    .service(nodes::delete_node)
            )
            .service(
                web::scope("/edge")
                    .service(web::scope("/metadata")
                        .service(edge_metadata::get_edge_metadata)
                        .service(edge_metadata::get_edge_metadata_singleton)
                        .service(edge_metadata::get_edge_metadata_singleton_key)
                        .service(edge_metadata::update_edge_metadata)
                        .service(edge_metadata::post_edge_metadata)
                        .service(edge_metadata::delete_edge_metadata)
                    )
                    .service(edges::get_edges)
                    .service(edges::get_edge)
                    .service(edges::post_edge)
                    .service(edges::update_edge)
                    .service(edges::delete_edge)
            )
    })
    .bind(format!("{}:{}", config.server_addr, args.port))?
    .run()
    .await
}
