use std::sync::{Mutex, Arc};
use std::thread;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

//use notify::{RecommendedWatcher, RecursiveMode, Result as Notify_Result, Watcher, watcher};
//use notify::{Watcher, RecommendedWatcher, RecursiveMode, Result};
use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};


//use notify::event::{EventKind, ModifyKind, Event};
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
    let current_dir = current_dir.to_str().unwrap();

    let (tx, rx) = channel();
//    let x = Watcher::new(tx, Duration::from_secs(2)).unwrap();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2)).unwrap();

    let x = watcher.watch(current_dir, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::NoticeWrite(f) => {
                        update_api_data(f.to_str().unwrap(), &current_dir, data.clone());
                    },
                    DebouncedEvent::Create(f) => {
                        update_api_data(f.to_str().unwrap(), &current_dir, data.clone());
                    },
                    DebouncedEvent::NoticeRemove(f) => {
                        update_api_data(f.to_str().unwrap(), &current_dir, data.clone());
                    },
                    DebouncedEvent::Rename(f1,f2) => {
                        update_api_data(f2.to_str().unwrap(), &current_dir, data.clone());
                    },
                    _ => {

                    }
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}


/// 发生文件改动/新增时，更新接口文档数据
/// README.md, json数据
fn update_api_data(filepath:&str, current_dir: &str, data: web::Data<Mutex<db::Database>>) {
    let mut api_docs: HashMap<String, db::ApiDoc> = HashMap::new();
    let mut api_data: HashMap<String, HashMap<String, Arc<Mutex<db::ApiData>>>> = HashMap::new();
    let mut fileindex_data: HashMap<String, HashSet<String>> = HashMap::new();

    let mut data = data.lock().unwrap();
    let filename = filepath.trim_start_matches(&format!("{}/", current_dir));

    if filename == "README.md" {
        let basic_data = db::load_basic_data();
        data.basic_data = basic_data;
    } else if  filename == "_settings.json" {
        // 全局重新加载
        *data = db::Database::load();
       return;
    } else if filename.contains("_data/") {
        // 如果修改的是_data里面的文件，需要通过fileindex_datal来找到对应文件更新
        match data.fileindex_data.get(filename) {
            Some(ref_files) => {
                // 把找到的文件全部重新load一遍
                for ref_file in ref_files {
                    db::Database::load_a_api_json_file(ref_file, &data.basic_data, &mut api_data, &mut api_docs, &mut fileindex_data);
                }
            }
            None => ()
        }
    } else {
        db::Database::load_a_api_json_file(filename, &data.basic_data, &mut api_data, &mut api_docs, &mut fileindex_data);
    }

    for (k, v) in api_data {
        data.api_data.insert(k, v);
    }

    for (k, v) in api_docs {
        data.api_docs.insert(k, v);

    }

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


pub fn get_random_chinese_chars(mut length: u32) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();
    while length > 0 {
        let n: u32 = rng.gen_range(0x4e00, 0x9fa5);
        let n: u32 = rng.gen_range(0x4e00, 0x9fa5);
        match char::from_u32(n) {
            Some(c) => {
                s.push(c);
                length -= 1;
            }
            None => continue
        }
    }
    s
}