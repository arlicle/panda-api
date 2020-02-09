use actix_web::{middleware, web, App, HttpServer};
use actix_files::Files;

use dotenv::dotenv;
use std::sync::Mutex;

mod db;
mod api;
mod utils;
mod websocket;
mod server;

mod mock;
mod client;

use structopt::StructOpt;
use actix::Actor;


#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Config {
    /// Listen ip
    #[structopt(short, long, default_value = "127.0.0.1", env = "PANDA_API_HOST")]
    host: String,

    /// Listen port
    #[structopt(short, long, default_value = "9000", env = "PANDA_API_PORT")]
    port: usize,

    /// create auth token length
    #[structopt(short, long, env = "PANDA_API_PORT")]
    token_length: Option<usize>,
}


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    dotenv().ok();
    pretty_env_logger::init();


    let conf = Config::from_args();
    if let Some(token_length) = conf.token_length {
        // create token
        for _ in 0..10 {
            println!("{}", mock::basic::string(token_length as u64, 0, 0));
        }
        return Ok(());
    }

    match dirs::home_dir() {
        Some(path) => {
            let current_dir = std::env::current_dir().expect("Failed to determine current directory");
            if path == current_dir {
                println!("You can not run panda api on double click, you need run it on shell with command at api docs folder. ex: ./panda , the more at https://github.com/arlicle/panda-api");
                return Ok(());
            }
        },
        None => println!("Impossible to get your home dir!"),
    }

    let db = db::Database::load();

    let websocket_api = &db.websocket_api.clone();
    let w = websocket_api.lock().unwrap();

    let websocket_uri = w.url.clone();
    let web_db = web::Data::new(Mutex::new(db));

    utils::watch_api_docs_change(web_db.clone());

    let server = server::ChatServer::default();
    let server = server.start();
    println!("Starting service on http://{}:{}", conf.host, conf.port);
    HttpServer::new(move || {
        App::new()
            .data(server.clone())
            .app_data(web_db.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Logger::new("%a %{User-Agent}i"))
            .wrap(middleware::DefaultHeaders::new()
                .header("Panda-Api", "0.5")
                .header("Access-Control-Allow-Headers", "*")
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "*"))

            .service(web::resource("/__api_docs/").route(web::get().to(api::get_api_doc_basic)))
            .service(web::resource("/__api_docs/api_data/").route(web::get().to(api::get_api_doc_data)))
            .service(web::resource("/__api_docs/_data/").route(web::get().to(api::get_api_doc_schema_data)))
            .service(web::resource("/").route(web::get().to(api::theme_view)))
            .service(web::resource("/static/*").route(web::get().to(api::theme_view)))
            .service(Files::new("/_upload", "_data/_upload"))

            .service(web::resource(&websocket_uri).to(api::chat_route))
            .service(web::resource("/*").to(api::action_handle))
    })
        .bind(format!("{}:{}", conf.host, conf.port))?
        .run()
        .await
}