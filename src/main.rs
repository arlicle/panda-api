use actix_web::{middleware, web, App, HttpServer};
use actix_files::Files;

use dotenv::dotenv;
use std::sync::{Mutex};

mod db;
mod api;
mod utils;
mod websocket;
mod server;

use structopt::StructOpt;


#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Config {
    /// Listen ip
    #[structopt(short, long, default_value = "127.0.0.1", env = "PANDA_API_HOST")]
    host: String,

    /// Listen port
    #[structopt(short, long, default_value = "9000", env = "PANDA_API_PORT")]
    port: usize,
}


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    dotenv().ok();
    pretty_env_logger::init();
    let conf = Config::from_args();
    let db = db::Database::load();
    let web_db = web::Data::new(Mutex::new(db));

    utils::watch_api_docs_change(web_db.clone());

    println!("Starting service on http://{}:{}", conf.host, conf.port);
    HttpServer::new(move || {
        App::new()
            .app_data(web_db.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Logger::new("%a %{User-Agent}i"))
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2").header("Access-Control-Allow-Origin", "*"))

            .service(web::resource("/__api_docs/").route(web::get().to(api::get_api_doc_basic)))
            .service(web::resource("/__api_docs/api_data/").route(web::get().to(api::get_api_doc_data)))
            .service(web::resource("/__api_docs/_data/").route(web::get().to(api::get_api_doc_schema_data)))
            .service(Files::new("/js", "_data/theme/js"))
            .service(Files::new("/css", "_data/theme/css"))
            .service(Files::new("/_upload", "_data/_upload"))
            .service(
                web::resource("/*")
                    .route(web::get().to(api::action_handle))
                    .route(web::post().to(api::action_handle))
                    .route(web::put().to(api::action_handle))
                    .route(web::delete().to(api::action_handle))
            )
    })
        .bind(format!("{}:{}", conf.host, conf.port))?
        .run()
        .await
}