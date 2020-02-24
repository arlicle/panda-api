#![allow(unused_must_use)]
use json5;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;

use crate::db;
use actix_web::web;
use std::sync::{Arc, Mutex};

use crate::Test;

/// 执行测试后端接口
pub async fn run_test(conf: Test, db_data: web::Data<Mutex<db::Database>>) {
    log::info!("start run test job {:?}", conf);
    let db_data = db_data.lock().unwrap();
    let db_api_data = &db_data.api_data;
    let db_api_docs = &db_data.api_docs;

    let mut server_url = "";
    if &conf.server_url != "" {
        server_url = &conf.server_url;
    } else if &conf.server != "" {
        let mut server_url_tmp: Option<&str> = None;
        if let Some(db_api_settings) = &db_data.settings {
            let pointer_str = format!("/servers/{}/url", conf.server);
            if let Some(url) = db_api_settings.pointer(&pointer_str) {
                server_url_tmp = url.as_str();
            }
        } else {
            log::error!("not found server set in _settings.json5");
            return;
        }

        if let Some(url) = server_url_tmp {
            server_url = url;
        } else {
            log::error!(
                "not found server {} with url set in _settings.json5",
                conf.server
            );
            return;
        }
    } else {
        log::error!("required arguments server or server_url were not provided");
        return;
    }

    // 获取服务器信息，如果获取不到就报错
    if let Some(docs) = &conf.docs {
        // 执行整个文档接口测试
        for doc_filename in docs {
            if let Some(a_api_doc) = db_api_docs.get(doc_filename) {
                for api in &a_api_doc.apis {
                    do_a_api_test(api, server_url).await;
                }
            }
        }
    }

    if &conf.url != "" {
        // 执行单个url测试
        if let Some(apis) = db_api_data.get(&conf.url) {
            // 获取到url对应的接口文档列表
            for api in apis {
                do_a_api_test(api, server_url).await;
            }
        } else {
            // 如果获取不到，那么就报错
            log::error!("cannot find api {}", &conf.url);
        }
    }
}

async fn do_a_api_test(api: &Arc<Mutex<db::ApiData>>, server_url: &str) {
    let api = api.lock().unwrap();

    let api_url = format!("{}{}", server_url, &api.url);
    if let Some(test_data) = api.test_data.as_array() {
        for a_data in test_data {
            let mut body_data: &Value = &Value::Null;
            if let Some(b) = a_data.get("body") {
                body_data = b;
            }

            let mut query: &Value = &Value::Null;
            if let Some(b) = a_data.get("query") {
                query = b;
            }

            let mut response: &Value = &Value::Null;
            if let Some(b) = a_data.get("response") {
                response = b;
            }

            if is_has_method(&api.method, "POST") {
                post(&api_url, body_data, response).await;
            }

            if is_has_method(&api.method, "GET") {
                get(&api_url, query, response).await;
            }
        }
    }
}

fn is_has_method(methods: &Vec<String>, method: &str) -> bool {
    if methods.contains(&"*".to_string()) || methods.contains(&method.to_string()) {
        return true;
    }
    false
}

/// 执行单个url接口测试
pub async fn get(url: &str, query_data: &Value, response: &Value) {
    let mut queries: Vec<String> = vec![];

    if let Some(q) = query_data.as_object() {
        for (k, v) in q {
            let v2 = match v {
                Value::String(v2) => v2.to_string(),
                Value::Number(v2) => json!(v2).to_string(),
                Value::Bool(v2) => "bool".to_string(),
                _ | Value::Null => "".to_string(),
            };
            queries.push(format!("{}={}", k, v2));
        }
    }

    let mut new_url = url.to_string();
    let s = queries.join("&");
    if queries.len() > 0 {
        if url.contains("?") {
            new_url = new_url + "&" + &s;
        } else {
            new_url = new_url + "?" + &s;
        }
    }
    println!("request query: {:?}", query_data);
    let resp = match reqwest::get(&new_url).await {
        Ok(r) => {
            if let Ok(s) = r.text().await {
                let y = match json5::from_str::<Value>(&s) {
                    Ok(v) => {
                        println!("response {:?}", v);
                    }
                    Err(e) => {
                        log::error!("error {:?}", e);
                    }
                };
            }
        }
        Err(e) => {
            log::error!("error {:?}", e);
        }
    };
}

pub async fn post(url: &str, body_data: &Value, response: &Value) {
    // url, method, body, json requet_data，response
    println!("request body: {:?}", body_data);
    let resp = match reqwest::Client::new()
        .post(url)
        .json(&json!(body_data))
        .send()
        .await
    {
        Ok(r) => {
            if let Ok(s) = r.text().await {
                let y = match json5::from_str::<Value>(&s) {
                    Ok(v) => {
                        println!("response {:?}", v);
                    }
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
}
