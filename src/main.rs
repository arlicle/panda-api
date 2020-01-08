use actix_web::{middleware, web, App, HttpServer};
use structopt::StructOpt;
use dotenv::dotenv;
use std::sync::{Mutex, Arc};
use std::thread;
//use panda_api::watch_api_docs_change;

mod db;
mod api;
mod utils;

use regex::Regex;

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

    println!("conf is {:?}", conf);
//    let db = std::fs::read_to_string("data/model.json").expect(&format!("Unable to read file: {}", conf.folder));
//    let data = Mutex::new(serde_json::from_str(&db).expect("Parse db file error"));
    let db_file = String::from("api_docs/post.json");
    let db = db::Database::load(&db_file);
//    println!("db {:?}", db.lock().unwrap());
    println!("Hello, world!");

//    let counter = Arc::new(Mutex::new(0));
//    let counter = Arc::new(0);
//    let mut handles = vec![];
//    for _ in 0..10 {
//        let counter = Arc::clone(&counter);
//        let handle = thread::spawn(move || {
////            let mut c = counter.lock().unwrap();
//            let mut c = counter;
////            *c += 1;
//            c += 1;
//
//            println!("counter is {:?}", c);
//        });
//        handles.push(handle);
//    }
//    for handle in handles {
//        handle.join().unwrap();
//    }

    let web_db = web::Data::new(Mutex::new(db));

    utils::watch_api_docs_change(web_db.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(web_db.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2").header("Access-Control-Allow-Origin", "*"))

            .service(web::resource("/index").route(web::get().to(api::server_info)))
            .service(web::resource("/__api_docs/").route(web::get().to(api::get_api_doc_basic)))
            .service(web::resource("/__api_docs/data/").route(web::get().to(api::get_api_doc_data)))
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
