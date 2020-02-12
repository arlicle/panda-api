use std::collections::HashMap;
use serde_json::{json, Value, Map};
use serde::{Deserialize, Serialize};
use json5;

use crate::db;
use actix_web::web;
use std::sync::{Mutex, Arc};

use crate::Test;


/// 执行测试后端接口
pub async fn run_test(conf: Test, db_data: web::Data<Mutex<db::Database>>) {
    log::info!("start run test job");
    let db_data = db_data.lock().unwrap();
    let db_api_data = &db_data.api_data;

    // 获取服务器信息，如果获取不到就报错
    let server_url = "http://localhost:9000";
    if &conf.url != "" {
        // 执行单个url测试
        if let Some(apis) = db_api_data.get(&conf.url) {
            // 获取到url对应的接口文档列表
            for api in apis {
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
        } else {
            // 如果获取不到，那么就报错
            log::error!("cannot find api {}", &conf.url);
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
    let mut queries:Vec<String> = vec![];

    if let Some(q) = query_data.as_object() {
        for (k, v) in q {
            let v2 = match v {
                Value::String(v2) => v2.to_string(),
                Value::Number(v2) => json!(v2).to_string(),
                Value::Bool(v2) => "bool".to_string(),
                _ | Value::Null => "".to_string()
            };
            queries.push(format!("{}={}", k,v2));
        }
    }

    let mut new_url = url.to_string();
    let s = queries.join("&");
    if queries.len() > 0 {
        println!("query: {:?}", query_data);

        if url.contains("?") {
            new_url = new_url + "&" + &s;
        } else {
            new_url = new_url + "?" + &s;
        }
    }

    let resp = match reqwest::get(&new_url)
        .await {
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
    println!("body: {:?}", body_data);
    let resp = match reqwest::Client::new().post(url).json(&json!(body_data)).send().await {
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