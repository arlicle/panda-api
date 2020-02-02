use std::sync::{Mutex, Arc};
use std::thread;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};

use actix_web::web;

use std::sync::mpsc::channel;
use crate::db;
use std::env;


/// 建立异步线程，监控文件改动，当改动的时候，就重新生成文件
pub fn watch_api_docs_change(data: web::Data<Mutex<db::Database>>) {
    let current_dir = env::current_dir().expect("Failed to determine current directory");
    let current_dir = current_dir.to_str().unwrap().to_string();

    thread::spawn(move || {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2)).unwrap();
        watcher.watch(&current_dir, RecursiveMode::Recursive).unwrap();
        loop {
            match rx.recv() {
                Ok(event) => {
                    match event {
                        DebouncedEvent::NoticeWrite(f) => {
                            update_api_data(f.to_str().unwrap(), &current_dir, data.clone());
                        }
                        DebouncedEvent::Create(f) => {
                            update_api_data(f.to_str().unwrap(), &current_dir, data.clone());
                        }
                        DebouncedEvent::NoticeRemove(f) => {
                            update_api_data(f.to_str().unwrap(), &current_dir, data.clone());
                        }
                        DebouncedEvent::Rename(_f1, f2) => {
                            update_api_data(f2.to_str().unwrap(), &current_dir, data.clone());
                        }
                        _ => {}
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });
}


/// 发生文件改动/新增时，更新接口文档数据
/// README.md, json数据
fn update_api_data(filepath: &str, current_dir: &str, data: web::Data<Mutex<db::Database>>) {
    let mut api_docs: HashMap<String, db::ApiDoc> = HashMap::new();
    let mut api_data: HashMap<String, HashMap<String, Arc<Mutex<db::ApiData>>>> = HashMap::new();
    let mut fileindex_data: HashMap<String, HashSet<String>> = HashMap::new();
    let websocket_api = Arc::new(Mutex::new(db::ApiData::default()));

    let mut data = data.lock().unwrap();
    let filename = filepath.trim_start_matches(&format!("{}/", current_dir));

    if filename == "README.md" {
        let basic_data = db::load_basic_data();
        data.basic_data = basic_data;
    } else if filename == "_settings.json" || filename == "_settings.json5" {
        // 全局重新加载
        *data = db::Database::load();
        return;
    } else if filename == "_auth.json" || filename == "_auth.json5" {
        // 加载auth
        let auth_data = db::load_auth_data();
        data.auth_doc = auth_data;
        return;
    } else if filename.contains("_data/") {
        // 如果修改的是_data里面的文件，需要通过fileindex_datal来找到对应文件更新
        match data.fileindex_data.get(filename) {
            Some(ref_files) => {
                // 把找到的文件全部重新load一遍
                for ref_file in ref_files {
                    db::Database::load_a_api_json_file(ref_file, &data.basic_data, &mut api_data, &mut api_docs, websocket_api.clone(), &mut fileindex_data);
                }
            }
            None => ()
        }
    } else {
        db::Database::load_a_api_json_file(filename, &data.basic_data, &mut api_data, &mut api_docs, websocket_api.clone(), &mut fileindex_data);
    }

    for (k, v) in api_data {
        data.api_data.insert(k, v);
    }

    for (k, v) in api_docs {
        data.api_docs.insert(k, v);
    }


    data.websocket_api = websocket_api;

    for (ref_file, doc_files) in fileindex_data {
        if &ref_file != "" {
            match data.fileindex_data.get_mut(&ref_file) {
                Some(x) => {
                    for f in doc_files {
                        x.insert(f);
                    }
                }
                None => {
                    data.fileindex_data.insert(ref_file, doc_files);
                }
            }
        }
    }
}