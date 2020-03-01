use actix::Actor;
use actix_web::{middleware, web, App, HttpServer};
use std::sync::Mutex;
use structopt::StructOpt;
mod api;
mod client;
mod db;
mod mock;
mod server;
mod utils;
mod websocket;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let conf = ApplicationArguments::from_args();
    let mut test_conf: Option<Test> = None;
    if let Some(command) = conf.command {
        match command {
            Command::Test(t) => {
                test_conf = Some(t);
            }
            Command::Token(t) => {
                // 测试正则生成字符串
                // generate token
                for _ in 0..t.num {
                    println!("{}", mock::basic::string(t.length as u64, 0, 0));
                }
                return Ok(());
            }
        }
    }

    match dirs::home_dir() {
        Some(path) => {
            let current_dir =
                std::env::current_dir().expect("Failed to determine current directory");
            if path == current_dir {
                log::error!("You can not run panda api on double click, you need run it on shell with command at api docs folder. ex: ./panda , the more at https://github.com/arlicle/panda-api");
                return Ok(());
            }
        }
        None => log::error!("Impossible to get your home dir!"),
    }

    let db = db::Database::load();

    let websocket_api = &db.websocket_api.clone();
    let w = websocket_api.lock().unwrap();

    let websocket_uri = w.url.clone();
    let web_db = web::Data::new(Mutex::new(db));

    if let Some(test_conf) = test_conf {
        client::test::run_test(test_conf, web_db.clone()).await;
        return Ok(());
    }

    utils::watch_api_docs_change(web_db.clone());

    let server = server::ChatServer::default();
    let server = server.start();

    HttpServer::new(move || {
        App::new()
            .data(server.clone())
            .app_data(web_db.clone())
            .wrap(
                middleware::Logger::default()
                    .exclude("/__api_docs/")
                    .exclude("/__api_docs/api_data/")
                    .exclude("/__api_docs/_data/")
                    .exclude("/__api_docs/theme/"),
            )
            //            .wrap(middleware::Logger::new("%a %{User-Agent}i"))
            .wrap(
                middleware::DefaultHeaders::new()
                    .header("Panda-Api", "0.5")
                    .header("Access-Control-Allow-Headers", "*")
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "*"),
            )
            .service(web::resource("/__api_docs/").route(web::get().to(api::get_api_doc_basic)))
            .service(
                web::resource("/__api_docs/api_data/").route(web::get().to(api::get_api_doc_data)),
            )
            .service(
                web::resource("/__api_docs/_data/")
                    .route(web::get().to(api::get_api_doc_schema_data)),
            )
            .service(web::resource("/__api_docs/theme/*").route(web::get().to(api::theme_view)))
            .service(web::resource("/").route(web::get().to(api::theme_view)))
            .service(web::resource("/media/*").route(web::get().to(api::static_file_view)))
            .service(web::resource("/_upload/*").route(web::get().to(api::upload_file_view)))
            .service(web::resource(&websocket_uri).to(api::chat_route))
            .service(web::resource("/*").to(api::action_handle))
    })
    .bind(format!("{}:{}", conf.host, conf.port))?
    .run()
    .await
}

#[derive(Debug, StructOpt)]
pub struct TimeInfo {
    /// minute (0 - 59)
    #[structopt(default_value = "5")]
    pub minute: usize,

    /// hour (0 - 23)
    #[structopt(default_value = "0")]
    pub hour: usize,

    /// day of month (1 - 31)
    #[structopt(default_value = "0")]
    pub day: usize,

    /// month (1 - 12)
    #[structopt(default_value = "0")]
    pub month: usize,
}

#[derive(Debug, StructOpt)]
pub struct Token {
    /// token num
    #[structopt(short, long, default_value = "10")]
    pub num: usize,

    /// token length
    #[structopt(short, long, default_value = "64")]
    pub length: usize,
}

#[derive(Debug, StructOpt)]
pub struct Test {
    /// test server name, config in _settings.json
    #[structopt(short, long, default_value = "")]
    pub server: String,

    /// test server address, config in _settings.json
    #[structopt(long, default_value = "")]
    pub server_url: String,

    /// api url
    #[structopt(short, long, default_value = "")]
    pub url: String,

    /// all api url
    #[structopt(short = "A", long)]
    pub all: bool,

    /// run cron job
    #[structopt(short, long)]
    pub cron: bool,

    /// api doc
    #[structopt(short, long)]
    pub docs: Option<Vec<String>>,

    #[structopt(flatten)]
    pub timeinfo: TimeInfo,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// generate random auth token
    #[structopt(name = "token")]
    Token(Token),
    /// Run the tests
    Test(Test),
}

/// Panda api command
#[derive(Debug, StructOpt)]
#[structopt(name = "classify")]
pub struct ApplicationArguments {
    /// Listen ip
    #[structopt(short, long, default_value = "127.0.0.1", env = "PANDA_API_HOST")]
    pub host: String,

    /// Listen port
    #[structopt(short, long, default_value = "9000", env = "PANDA_API_PORT")]
    pub port: usize,

    #[structopt(subcommand)]
    pub command: Option<Command>,
}
