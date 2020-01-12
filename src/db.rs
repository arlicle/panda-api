use log::debug;
use serde_json::{json, Value, Map};
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
    pub fn load() -> Database {
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

        let doc_file_obj = v.as_object().unwrap();
        let doc_name = match doc_file_obj.get("name") {
            Some(name) => name.as_str().unwrap(),
            None => doc_file
        };
        let doc_name = doc_name.to_string();

        let doc_desc = match doc_file_obj.get("desc") {
            Some(desc) => desc.as_str().unwrap(),
            None => ""
        };
        let doc_desc = doc_desc.to_string();

        let doc_order: i64 = match doc_file_obj.get("order") {
            Some(order) => order.as_i64().expect("order is not number"),
            None => 0
        };

        let apis = match doc_file_obj.get("api") {
            Some(api) => api,
            None => { return; }
        };

        let mut api_vec = Vec::new();
        if let Some(api_array) = apis.as_array() {
            let mut ref_data;
            for api in api_array {
                ref_data = Value::Null;
                match api.get("$ref") {
                    // 处理api数据引用
                    Some(v) => {
                        let v = v.as_str().unwrap();
                        if let Some(value) = load_ref_file_data(v) {
                            ref_data = value;
                        }
                    }
                    None => ()
                }

                let name = match api.get("name") {
                    Some(name) => name.as_str().unwrap().to_string(),
                    None => {
                        match ref_data.get("name") {
                            Some(v) => v.as_str().unwrap().to_string(),
                            None => continue
                        }
                    }
                };

                let desc = get_api_field_value("desc", "".to_string(), api, &ref_data);
                let url = get_api_field_value("url", "".to_string(), api, &ref_data);
                let method = get_api_field_value("method", "GET".to_string(), api, &ref_data);
                let body_mode = get_api_field_value("body_mode", "json".to_string(), api, &ref_data);
//                let body = get_api_value("body", "json".to_string(), api, &ref_data);


                let body = match api.get("body") {
                    Some(body) => body.clone(),
                    None => {
                        match ref_data.get("body") {
                            Some(v) => v.clone(),
                            None => Value::Null
                        }
                    }
                };
                let body = parse_attribute_ref_value(body, doc_file_obj);


                let response = match api.get("response") {
                    Some(response) => {
                        response.clone()
                    }
                    None => {
                        match ref_data.get("response") {
                            Some(v) => v.clone(),
                            None => Value::Null
                        }
                    }
                };

                // 处理response中的$ref
                let response = parse_attribute_ref_value(response, doc_file_obj);


                let test_data = match api.get("test_data") {
                    Some(test_data) => {
//                        let a = match test_data.as_array().expect(&format!("json file {} test_data is not a array", doc_file));
                        test_data.clone()
                    }
                    None => {
                        match ref_data.get("test_data") {
                            Some(v) => v.clone(),
                            None => Value::Null
                        }
                    }
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
        }


        let api_doc = ApiDoc { name: doc_name, desc: doc_desc, order: doc_order, filename: doc_file.to_string(), apis: api_vec };


        api_docs.insert(doc_file.to_string(), api_doc);
    }
}


fn load_ref_file_data(ref_file: &str) -> Option<Value> {
    let ref_info: Vec<&str> = ref_file.split(":").collect();
    match ref_info.get(0) {
        Some(filename) => {
            // 加载数据文件
            if let Ok(d) = fs::read_to_string(format!("api_docs/{}", filename.trim_start_matches("./"))) {
                let d = fix_json(d);
                let mut data: Value = match serde_json::from_str(&d) {
                    Ok(v) => v,
                    Err(e) => {
                        println!("Parse json file {} error : {:?}", filename, e);
                        return None;
                    }
                };

                if let Some(key) = ref_info.get(1) {
                    if let Some(v) = data.pointer(&format!("/{}", &key.replace(".", "/"))) {
                        return Some(v.clone());
                    }
                }
            }
        }
        None => ()
    };
    None
}


/// 获取api里面字段的数据
/// 如 url, name, method等
fn get_api_field_value(key: &str, default_value: String, api: &Value, ref_data: &Value) -> String {
    match api.get(key) {
        Some(d) => d.as_str().unwrap().to_string(),
        None => {
            if let Some(v) = ref_data.get(key) {
                v.as_str().unwrap().to_string()
            } else {
                default_value
            }
        }
    }
}


/// parse $ref引用数据
fn parse_attribute_ref_value(value: Value, doc_file_obj: &Map<String, Value>) -> Value {
    if value.is_null() {
        return value;
    }

    if value.is_object() {
        let mut result: Map<String, Value> = Map::new();
        let value_obj = value.as_object().unwrap();
        let mut new_value = value_obj.clone();

        match value_obj.get("$ref") {
            Some(ref_val) => {
                let mut v_str = ref_val.as_str().unwrap();
                if v_str.contains("$") {
                    match doc_file_obj.get("define") {
                        Some(v2) => {
                            match v2.get(v_str.trim_start_matches("$")) {
                                Some(v3) => {
                                    v_str = v3.as_str().unwrap();
                                }
                                None => ()
                            }
                        }
                        None => ()
                    }
                }
                match load_ref_file_data(v_str) {
                    Some(vv) => {
                        new_value = vv.as_object().unwrap().clone();
                    }
                    None => ()
                }
                // 移除exclude中的字段
                match value_obj.get("$exclude") {
                    Some(e) => {
                        for v2 in e.as_array().unwrap() {
                            new_value.remove(v2.as_str().unwrap());
                        }
                    }
                    None => ()
                }
            }
            None => ()
        }

        for (k, v) in value_obj {
//            if k == "$ref" || k == "$exclude" {
//                continue;
//            } else {
                new_value.insert(k.to_string(), parse_attribute_ref_value(v.clone(), doc_file_obj));
//            }
        }
        return Value::Object(new_value);
    }

    value
}