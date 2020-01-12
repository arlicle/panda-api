use actix_web::{middleware, web, App, HttpServer};
use actix_files::Files;

use structopt::StructOpt;
use dotenv::dotenv;
use std::sync::{Mutex, Arc};
use std::thread;
//use panda_api::watch_api_docs_change;
use actix_web::dev::ResourceDef;
use std::char;

mod db;
mod api;
mod utils;

use regex::Regex;
use rand::{thread_rng, Rng};


#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Config {
    /// data folder
    #[structopt(short, long, default_value = "data")]
    folder: String,

    /// Listen ip
    #[structopt(long, default_value = "127.0.0.1", env = "MOCKRS_HOST")]
    host: String,

    /// Listen port
    #[structopt(long, default_value = "9000", env = "MOCKRS_PORT")]
    port: usize,
}


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    pretty_env_logger::init();
    let conf: Config = Config::from_args();

    let db = db::Database::load();

    let web_db = web::Data::new(Mutex::new(db));

    utils::watch_api_docs_change(web_db.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(web_db.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2").header("Access-Control-Allow-Origin", "*"))

            .service(web::resource("/index").route(web::get().to(api::server_info)))
            .service(web::resource("/__api_docs/").route(web::get().to(api::get_api_doc_basic)))
            .service(web::resource("/__api_docs/api_data/").route(web::get().to(api::get_api_doc_data)))
            .service(web::resource("/__api_docs/_data/").route(web::get().to(api::get_api_doc_schema_data)))
            .service(Files::new("/js", "theme/js"))
            .service(Files::new("/css", "theme/css"))
            .service(
                web::resource("/*")
                    .route(web::get().to(api::do_get))
                    .route(web::post().to(api::do_post)),
            )
    })
        .bind(format!("{}:{}", conf.host, conf.port))?
        .run()
        .await
}
