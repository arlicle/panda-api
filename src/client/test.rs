use std::collections::HashMap;
use serde_json::{json, Value, Map};
use serde::{Deserialize, Serialize};
use json5;



pub async fn do_test_case_action() {

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