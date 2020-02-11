use std::collections::HashMap;
use serde_json::{json, Value, Map};
use serde::{Deserialize, Serialize};
use json5;

use crate::db;
use actix_web::web;
use std::sync::{Mutex, Arc};

use crate::Test;

/// 执行测试后端接口
pub async fn run_test(conf:Test, db_data: web::Data<Mutex<db::Database>>) {
    let db_data = db_data.lock().unwrap();
    let db_api_data = &db_data.api_data;

    // 检查服务器是否在配置中

    if &conf.url != "" {
        // 执行单个url测试
        if let Some(apis) = db_api_data.get(&conf.url) {
            // 获取到url对应的接口文档列表
        } else {
            // 如果获取不到，那么就报错
        }
    }
}


/// 执行单个url接口测试
pub async fn run_a_url_test() {

}

pub async fn post() {

    println!("66666");

    let resp = match reqwest::get("http://localhost:9000/login/").await {
        Ok(r) => {
            if let Ok(s) = r.text().await {
                println!("jjjjjj");
                println!("s {}", s);
                let y = match json5::from_str::<Value>(&s) {
                    Ok(v) => {
                        println!("v {:?}", v);
                    },
                    Err(e) => {
                        println!("sss {:?}", e);
                    }
                };
            }
        }
        Err(e) => {
            println!("error {:?}", e);
        }

    };
    println!("{:#?}", resp);


    let resp = match reqwest::Client::new().post("http://localhost:9000/login/").json(&json!({"username":"root", "password":"123"})).send().await {
        Ok(r) => {
            if let Ok(s) = r.text().await {
                println!("jjjjjj");
                println!("s {}", s);
                let y = match json5::from_str::<Value>(&s) {
                    Ok(v) => {
                        println!("v {:?}", v);
                    },
                    Err(e) => {
                        println!("sss {:?}", e);
                    }
                };
            }
        }
        Err(e) => {
            println!("error {:?}", e);
        }

    };
    println!("{:#?}", resp);


}