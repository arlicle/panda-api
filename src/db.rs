use log::debug;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};

use std::fs;
use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use regex::Regex;
use walkdir::WalkDir;


#[derive(Debug)]
pub struct Database {
    pub basic_data: BasicData,
    pub api_docs: HashMap<String, ApiDoc>,
    // filename => apidoc
    pub api_data: HashMap<String, HashMap<String, Arc<Mutex<ApiData>>>>, // url => api data request
}


#[derive(Debug)]
pub struct BasicData {
    pub read_me: String,
    pub project_name: String,
    pub project_desc: String,

}

#[derive(Debug)]
pub struct ApiDoc {
    pub name: String,
    pub desc: String,
    pub order: i64,
    pub filename: String,
    pub apis: Vec<Arc<Mutex<ApiData>>>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiData {
    pub name: String,
    pub desc: String,
    pub url: String,
    pub method: String,
    pub body_mode: String,
    pub body: Value,
    pub response: Value,
    pub test_data: Value,
}


fn fix_json(org_string: String) -> String {
    let re = Regex::new(r#":\s*"[\s\S]*?\n*[\s\S]*?""#).unwrap();
    let mut new_string = org_string.clone();
    for cap in re.captures_iter(&org_string) {
        let x = &cap[0];
        if x.contains("\n") {
            let y = x.replace("\n", r#"\n"#);
            new_string = new_string.replace(x, &y);
        }
    }
    new_string
}


pub fn load_basic_data() -> BasicData {
    let read_me = match fs::read_to_string("api_docs/README.md") {
        Ok(x) => x,
        Err(_) => "Panda api docs".to_string()
    };

    let f = "api_docs/_settings.json";
    let d = fs::read_to_string(f).expect(&format!("Unable to read file: {}", f));
    let d = fix_json(d);
    let mut v: Value = serde_json::from_str(&d).expect(&format!("Parse json file {} error", f));

    let obj = v.as_object().unwrap();

    let project_name = match obj.get("project_name") {
        Some(name) => name.as_str().unwrap(),
        None => "Panda api docs"
    };
    let project_name = project_name.to_string();

    let project_desc = match obj.get("project_desc") {
        Some(name) => name.as_str().unwrap(),
        None => ""
    };
    let project_desc = project_desc.to_string();

    BasicData { read_me, project_name, project_desc }
}


impl Database {
    /// 加载api docs 接口的json数据、配置、相关文档
    pub fn load(file: &String) -> Database {
        let basic_data = load_basic_data();

        let mut api_docs = HashMap::new();
        let mut api_data: HashMap<String, HashMap<String, Arc<Mutex<ApiData>>>> = HashMap::new();

        for entry in WalkDir::new("api_docs") {
            let e = entry.unwrap();
            let doc_file = e.path().to_str().unwrap();

            Self::load_a_api_json_file(doc_file, &mut api_data, &mut api_docs);
        }

//        let api_data = Mutex::new(api_data);
//        let docs_data = Mutex::new(docs_data);
        Database { basic_data, api_data, api_docs }
    }


    /// 只加载一个api_doc文件的数据
    ///
    pub fn load_a_api_json_file(doc_file: &str, api_data: &mut HashMap<String, HashMap<String, Arc<Mutex<ApiData>>>>, api_docs: &mut HashMap<String, ApiDoc>) {
        if !doc_file.ends_with(".json") || doc_file.ends_with("_settings.json") || doc_file.contains("/_data/") {
            return;
        }

        let d = fs::read_to_string(doc_file).expect(&format!("Unable to read file: {}", doc_file));
        let d = fix_json(d);
        let mut v: Value = match serde_json::from_str(&d) {
            Ok(v) => v,
            Err(e) => {
                println!("Parse json file {} error : {:?}", doc_file, e);
                return;
            }
        };

        let obj = v.as_object().unwrap();
        let doc_name = match obj.get("name") {
            Some(name) => name.as_str().unwrap(),
            None => doc_file
        };
        let doc_name = doc_name.to_string();

        let doc_desc = match obj.get("desc") {
            Some(desc) => desc.as_str().unwrap(),
            None => ""
        };
        let doc_desc = doc_desc.to_string();

        let doc_order: i64 = match obj.get("order") {
            Some(order) => order.as_i64().expect("order is not number"),
            None => 0
        };

        let apis = match obj.get("api") {
            Some(api) => api,
            None => { return; }
        };

        let mut api_vec = Vec::new();
        for api in apis.as_array().unwrap() {
            let name = api.get("name").unwrap().as_str().unwrap().to_string();
            let desc = api.get("desc").unwrap().as_str().unwrap().to_string();
            let url = api.get("url").unwrap().as_str().unwrap().to_string();

            let method = match api.get("method") {
                Some(method) => method.as_str().unwrap().to_uppercase(),
                None => "GET".to_string()
            };
            let body_mode = match api.get("body_mode") {
                Some(body_mode) => body_mode.as_str().unwrap().to_lowercase(),
                None => "json".to_string()
            };
            let body = match api.get("body") {
                Some(body) => body.clone(),
                None => Value::Null
            };

            let response = match api.get("response") {
                Some(response) => {
                    response.clone()
                }
                None => Value::Null
            };

            let test_data = match api.get("test_data") {
                Some(test_data) => {
                    let a = test_data.as_array().expect(&format!("json file {} test_data is not a array", doc_file));
                    test_data.clone()
                }
                None => Value::Null
            };


            let a_api_data = Arc::new(Mutex::new(ApiData { name, desc, body_mode, body, response, test_data, url: url.clone(), method: method.clone() }));
            // 形成 { url: {method:api} }
            match api_data.get_mut(&url) {
                Some(mut data) => {
                    data.insert(method.clone(), a_api_data.clone());
                }
                None => {
                    let mut x = HashMap::new();
                    x.insert(method.clone(), a_api_data.clone());
                    api_data.insert(url.clone(), x);
                }
            }
//                    api_data.insert(url.clone(), api.clone());
            api_vec.push(a_api_data.clone());
        }


        let api_doc = ApiDoc { name: doc_name, desc: doc_desc, order: doc_order, filename: doc_file.to_string(), apis: api_vec };


        api_docs.insert(doc_file.to_string(), api_doc);
    }
}
