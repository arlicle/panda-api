use std::sync::{Mutex, Arc};
use std::thread;
use std::collections::HashMap;
use std::time::Duration;
use notify::{RecommendedWatcher, RecursiveMode, Result as Notify_Result, Watcher, watcher, Config};
use notify::event::{EventKind, ModifyKind, Event};
use crossbeam_channel::unbounded;
use actix_web::{http, web, HttpRequest, HttpResponse};

use std::sync::mpsc::channel;
use crate::db;
use std::env;
use std::char;
use rand::{thread_rng, Rng};


/// 建立异步线程，监控文件改动，当改动的时候，就重新生成文件
pub fn watch_api_docs_change(data: web::Data<Mutex<db::Database>>) {
    let current_dir = env::current_dir().expect("Failed to determine current directory");
    let current_dir = current_dir.to_str().unwrap().to_string();


    thread::spawn(move || {
        let (sender, receiver) = unbounded();

        let mut watcher = watcher(sender, Duration::from_secs(1)).unwrap();
        watcher.watch("api_docs", RecursiveMode::Recursive).unwrap();

        loop {
            match receiver.recv() {
                Ok(event) => {
                    match event {
                        Ok(e) => match e.kind {
                            EventKind::Modify(_) => {
                                if let Some(_) = e.flag() {
                                    update_api_data(e, &current_dir, data.clone());
                                }
                            }
                            EventKind::Create(_) => {
                                update_api_data(e, &current_dir, data.clone());
                            }
                            EventKind::Remove(_) => {
                                if let Some(_) = e.flag() {
                                    let mut data = data.lock().unwrap();

                                    for file_path in e.paths.iter() {
                                        let filename = file_path.to_str().unwrap().trim_start_matches(&format!("{}/", current_dir));
                                        data.api_data.remove(filename);
                                        data.api_docs.remove(filename);
                                    }
                                }
                            }
                            // other do nothing
                            _ => (),
                        }
                        Err(e) => println!("event error {:?}", e),
                    }
                }
                Err(err) => println!("watch error: {:?}", err),
            }
        };
    });
}


/// 发生文件改动/新增时，更新接口文档数据
/// README.md, json数据
fn update_api_data(e: Event, current_dir: &str, data: web::Data<Mutex<db::Database>>) {
    let mut api_docs: HashMap<String, db::ApiDoc> = HashMap::new();
    let mut api_data: HashMap<String, HashMap<String, Arc<Mutex<db::ApiData>>>> = HashMap::new();
    let mut fileindex_data: HashMap<String, Vec<String>> = HashMap::new();

    let mut data = data.lock().unwrap();
    for file_path in e.paths.iter() {
        let filename = file_path.to_str().unwrap().trim_start_matches(&format!("{}/", current_dir));
        if filename == "api_docs/README.md" || filename == "api_docs/_settings.json" {
            let basic_data = db::load_basic_data();

            data.basic_data = basic_data;
        } else {
            db::Database::load_a_api_json_file(filename, &mut api_data, &mut api_docs, &mut fileindex_data);
        }
    }

    for (k, v) in api_data {
        data.api_data.insert(k, v);
    }

    for (k, v) in api_docs {
        data.api_docs.insert(k, v);
    }
}


pub fn get_random_chinese_chars(mut length: u32) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();
    while length > 0 {
        let n: u32 = rng.gen_range(0x4e00, 0x9fa5);
        let n: u32 = rng.gen_range(0x4e00, 0x9fa5);
        println!("n is {} {:?}", n, char::from_u32(n));
        match char::from_u32(n) {
            Some(c) => {
                s.push(c);
                length -= 1;
            },
            None => continue
        }
    }
    s
}