use serde_json::{json, Value, Map};
use serde::{Deserialize, Serialize};
use json5;
use std::fs;
use std::sync::{Mutex, Arc};
use std::collections::{HashMap, HashSet};
use regex::Regex;
use walkdir::WalkDir;
use std::path::Path;

#[derive(Debug)]
pub struct Database {
    pub basic_data: BasicData,
    pub api_docs: HashMap<String, ApiDoc>,
    pub api_data: HashMap<String, HashMap<String, Arc<Mutex<ApiData>>>>,
    pub fileindex_data: HashMap<String, HashSet<String>>,
}


#[derive(Debug)]
pub struct BasicData {
    pub read_me: String,
    pub project_name: String,
    pub project_desc: String,
    pub global_value: Value,
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
    pub auth: bool,
    pub body_mode: String,
    pub body: Value,
    pub query: Value,
    pub response: Value,
    pub test_data: Value,
}


fn fix_json(org_string: String) -> String {
    let re = Regex::new(r#":\s*"[\s\S]*?\n*[\s\S]*?""#).unwrap(); // 修复字符串换行

    let re3 = Regex::new(r"/\*(.|[\r\n])*?\*/").unwrap(); // 去掉/* */注释

    let mut new_string = org_string.clone();
    for cap in re.captures_iter(&org_string) {
        let x = &cap[0];
        if x.contains("\n") {
            let y = x.replace("\n", r#"\n"#);
            new_string = new_string.replace(x, &y);
        }
    }

    let new_string = re3.replace_all(&new_string, "").to_string();

    new_string
}


pub fn load_basic_data() -> BasicData {
    let read_me = match fs::read_to_string("README.md") {
        Ok(x) => x,
        Err(_) => "Panda api docs".to_string()
    };

    let settings_files = ["_settings.json5", "_settings.json"];

    let mut setting_value = json!({});
    for settings_file in settings_files.iter() {
        match fs::read_to_string(settings_file) {
            Ok(v) => {
                let v = fix_json(v);
                match json5::from_str(&v) {
                    Ok(v) => v,
                    Err(e) => {
//                        println!("Parse json file {} error {:?}", settings_file, e);
                        json!({})
                    }
                }
            }
            Err(_) => {
//                println!("warning: no '{}' file", settings_file);
                json!({})
            }
        };
    }


    let obj = setting_value.as_object().unwrap();

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

    let global_value = match obj.get("global") {
        Some(v) => v.clone(),
        None => Value::Null
    };

    BasicData { read_me, project_name, project_desc, global_value }
}


impl Database {
    /// 加载api docs 接口的json数据、配置、相关文档
    pub fn load() -> Database {
        let basic_data = load_basic_data();

        let mut api_docs = HashMap::new();
        let mut api_data: HashMap<String, HashMap<String, Arc<Mutex<ApiData>>>> = HashMap::new();
        let mut fileindex_data: HashMap<String, HashSet<String>> = HashMap::new();

        for entry in WalkDir::new("./") {
            let e = entry.unwrap();
            let doc_file = e.path().to_str().unwrap().trim_start_matches("./");
            Self::load_a_api_json_file(doc_file, &basic_data, &mut api_data, &mut api_docs, &mut fileindex_data);
        }

        Database { basic_data, api_data, api_docs, fileindex_data }
    }


    /// 只加载一个api_doc文件的数据
    ///
    pub fn load_a_api_json_file(doc_file: &str, basic_data: &BasicData, api_data: &mut HashMap<String, HashMap<String, Arc<Mutex<ApiData>>>>, api_docs: &mut HashMap<String, ApiDoc>, fileindex_data: &mut HashMap<String, HashSet<String>>) {
        if !(doc_file.ends_with(".json") || doc_file.ends_with(".json5")) || doc_file == "_settings.json" || doc_file == "_settings.json5" || doc_file.contains("_data/") || doc_file.starts_with(".") {
            return;
        }

        let d = match fs::read_to_string(doc_file) {
            Ok(d) => d,
            Err(e) => {
                println!("Unable to read file: {} {:?}", doc_file, e);
                return;
            }
        };

        let d = fix_json(d);
        let json_value: Value = match json5::from_str(&d) {
            Ok(v) => v,
            Err(e) => {
                println!("Parse json file {} error : {:?}", doc_file, e);
                return;
            }
        };

        let doc_file_obj = json_value.as_object().unwrap();
        let doc_name = match doc_file_obj.get("name") {
            Some(name) => {
                match name.as_str() {
                    Some(v) => v.to_string(),
                    None => format!("{}", name)
                }
            }
            None => doc_file.to_string()
        };

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
            Some(api) => api.clone(),
            None => { json!([]) }
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
                        let (ref_file, ref_data2) = load_ref_file_data(v, doc_file);
                        if ref_file != "" {
                            match fileindex_data.get_mut(&ref_file) {
                                Some(x) => {
                                    x.insert(doc_file.to_string());
                                }
                                None => {
                                    let mut b = HashSet::new();
                                    b.insert(doc_file.to_string());
                                    fileindex_data.insert(ref_file, b);
                                }
                            }
                        }

                        if let Some(value) = ref_data2 {
                            ref_data = value;
                        }
                    }
                    None => ()
                }

                let name = get_api_field_string_value("name", doc_file.to_string(), api, &ref_data, &basic_data.global_value);
                let desc = get_api_field_string_value("desc", "".to_string(), api, &ref_data, &basic_data.global_value);
                let url = get_api_field_string_value("url", "".to_string(), api, &ref_data, &basic_data.global_value);
                let method = get_api_field_string_value("method", "GET".to_string(), api, &ref_data, &basic_data.global_value);
                let body_mode = get_api_field_string_value("body_mode", "json".to_string(), api, &ref_data, &basic_data.global_value);
                let auth = get_api_field_bool_value("auth", false, api, &ref_data, &basic_data.global_value);
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
                let (mut ref_files, body) = parse_attribute_ref_value(body, doc_file_obj, doc_file);

                let query = match api.get("query") {
                    Some(query) => query.clone(),
                    None => {
                        match ref_data.get("query") {
                            Some(v) => v.clone(),
                            None => Value::Null
                        }
                    }
                };
                let (mut ref_files2, query) = parse_attribute_ref_value(query, doc_file_obj, doc_file);
                ref_files.append(&mut ref_files2);

                // 最后查询global_value
                let mut response: Map<String, Value> = match basic_data.global_value.pointer("/api/response") {
                    Some(v) => {
                        v.as_object().unwrap().clone()
                    }
                    None => json!({}).as_object().unwrap().clone()
                };


                if let Some(r) = ref_data.get("response") {
                    if let Some(rm) = r.as_object() {
                        for (k, v) in rm {
                            response.insert(k.to_string(), v.clone());
                        }
                    }
                }

                if let Some(r) = api.get("response") {
                    if let Some(rm) = r.as_object() {
                        for (k, v) in rm {
                            response.insert(k.to_string(), v.clone());
                        }
                    }
                }

                // 处理response中的$ref
                let (mut ref_files2, response) = parse_attribute_ref_value(Value::Object(response), doc_file_obj, doc_file);

                ref_files.append(&mut ref_files2);
                for ref_file in ref_files {
                    if &ref_file != "" {
                        match fileindex_data.get_mut(&ref_file) {
                            Some(x) => {
                                x.insert(doc_file.to_string());
                            }
                            None => {
                                let mut b = HashSet::new();
                                b.insert(doc_file.to_string());
                                fileindex_data.insert(ref_file, b);
                            }
                        }
                    }
                }


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


                let a_api_data = Arc::new(Mutex::new(ApiData { name, desc, body_mode, body, query, response, test_data, auth: auth, url: url.clone(), method: method.clone() }));
                // 形成 { url: {method:api} }
                match api_data.get_mut(&url) {
                    Some(data) => {
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


fn load_ref_file_data(ref_file: &str, doc_file: &str) -> (String, Option<Value>) {
    let ref_info: Vec<&str> = ref_file.split(":").collect();


    match ref_info.get(0) {
        Some(filename) => {
            let mut file_path;
            if filename.starts_with("./_data") {
                let path = Path::new(doc_file).parent().unwrap();
                file_path = format!("{}/{}", path.to_str().unwrap(), filename.trim_start_matches("./"));
            } else if filename.starts_with("/_data") {
                file_path = filename.trim_start_matches("/").to_string();
            } else {
                file_path = filename.to_string();
            }
            file_path = file_path.trim_start_matches("/").to_string();
            // 加载数据文件
            if let Ok(d) = fs::read_to_string(&file_path) {
                let d = fix_json(d);
                let data: Value = match json5::from_str(&d) {
                    Ok(v) => v,
                    Err(e) => {
                        println!("Parse json file {} error : {:?}", filename, e);
                        return ("".to_string(), None);
                    }
                };

                if let Some(key) = ref_info.get(1) {
                    if let Some(v) = data.pointer(&format!("/{}", &key.replace(".", "/"))) {
                        return (file_path, Some(v.clone()));
                    }
                }
            } else {
                println!("file {} not found", &file_path);
                return (file_path, None);
            }
        }
        None => ()
    };
    ("".to_string(), None)
}


/// 获取api里面字段的数据
/// 如 url, name, method等
fn get_api_field_string_value(key: &str, default_value: String, api: &Value, ref_data: &Value, global_data: &Value) -> String {
    match api.get(key) {
        Some(d) => {
            if let Some(v) = d.as_str() {
                return v.to_owned();
            } else {
                return format!("{}", d);
            }
        }
        None => ()
    }
    if let Some(d) = ref_data.get(key) {
        if let Some(v) = d.as_str() {
            return v.to_owned();
        } else {
            return format!("{}", d);
        }
    }

    // 最后查询global_value
    match global_data.get("api") {
        Some(v) => {
            match v.get(key) {
                Some(v2) => {
                    if let Some(d) = v2.as_str() {
                        return d.to_owned();
                    } else {
                        return format!("{}", v2);
                    }
                }
                None => ()
            }
        }
        None => ()
    }
    default_value
}


fn get_api_field_bool_value(key: &str, default_value: bool, api: &Value, ref_data: &Value, global_data: &Value) -> bool {
    match api.get(key) {
        Some(d) => {
            if let Some(v) = d.as_bool() {
                return v;
            } else {
                println!("{} value is not a bool", key)
            }
        }
        None => ()
    }

    if let Some(d) = ref_data.get(key) {
        if let Some(v) = d.as_bool() {
            return v;
        } else {
            println!("{} value is not a bool", key)
        }
    }

    match global_data.get("api") {
        Some(v) => {
            match v.get(key) {
                Some(d) => {
                    if let Some(v2) = d.as_bool() {
                        return v2;
                    } else {
                        println!("{} value is not a bool", key)
                    }
                }
                None => ()
            }
        }
        None => ()
    }

    default_value
}

/// parse $ref引用数据
fn parse_attribute_ref_value(value: Value, doc_file_obj: &Map<String, Value>, doc_file: &str) -> (Vec<String>, Value) {
    let mut ref_files: Vec<String> = Vec::new();
    if value.is_null() {
        return (ref_files, value);
    }

    if value.is_object() {
        let value_obj = value.as_object().unwrap();
        let mut new_value = value_obj.clone();

        if let Some(ref_val) = value_obj.get("$ref") {
            let mut v_str = ref_val.as_str().unwrap();
            let mut new_v_str = "".to_string();
            if v_str.contains("$") {
                match doc_file_obj.get("define") {
                    Some(defined) => {
                        let re = Regex::new(r"\$\w+").unwrap();
                        match re.find(v_str) {
                            Some(m) => {
                                let m_str = &v_str[m.start() + 1..m.end()];
                                match defined.get(m_str) {
                                    Some(v3) => {
                                        new_v_str = format!("{}{}", v3.as_str().unwrap(), &v_str[m.end()..]);
                                    }
                                    None => ()
                                }
                            }
                            None => ()
                        };
                    }
                    None => ()
                }
            }
            if new_v_str != "".to_string() {
                v_str = new_v_str.as_str();
            }
            let (ref_file, ref_data) = load_ref_file_data(v_str, doc_file);
            ref_files.push(ref_file);
            let mut has_include = false;
            if let Some(vv) = ref_data {
                new_value = match vv.as_object() {
                    Some(ref_data_map) => {
                        // 判断是否有include 字段，然后只引入include
                        let mut new_result = Map::new();
                        if let Some(e) = value_obj.get("$include") {
                            for v2 in e.as_array().unwrap() {
                                has_include = true;
                                let key_str = v2.as_str().unwrap();
                                if let Some(v) = ref_data_map.get(key_str) {
                                    new_result.insert(key_str.to_string(), v.clone());
                                }
                            }
                        }
                        if has_include {
                            new_result
                        } else {
                            ref_data_map.clone()
                        }
                    }
                    None => {
                        println!(" file value error '{}' got {:?}", v_str, vv);
                        json!({}).as_object().unwrap().clone()
                    }
                }
            }

            if !has_include {
                // 移除exclude中的字段
                if let Some(e) = value_obj.get("$exclude") {
                    for v2 in e.as_array().unwrap() {
                        let key_str = v2.as_str().unwrap();
                        if key_str.contains(".") {
                            // 如果exclude中含有.点，表示要嵌套的去移除字段
                        } else {
                            new_value.remove(key_str);
                        }
                    }
                }
            }
        }


        for (k, v) in value_obj {
            if v.is_string() && v.as_str().unwrap() == "$del" {
                new_value.remove(k);
                continue;
            } else if k == "$ref" || k == "$exclude" || k == "$include" {
                continue;
            } else {
                let (mut ref_files2, field_value) = parse_attribute_ref_value(v.clone(), doc_file_obj, doc_file);
                ref_files.append(&mut ref_files2);
                new_value.insert(k.to_string(), field_value);
            }
        }

        return (ref_files, Value::Object(new_value));
    } else if value.is_array() {
        // 处理array
        if let Some(value_array) = value.as_array() {
            if let Some(value_array_one) = value_array.get(0) {
                let (ref_files, array_item_value) = parse_attribute_ref_value(value_array_one.clone(), doc_file_obj, doc_file);
                return (ref_files, Value::Array(vec![array_item_value]));
            } else {
                println!(" file array value empty '{}' got {:?}", doc_file, value);
            }
        }
    }

    (ref_files, value)
}

/// 可以嵌套的删除Value里面的某一个字段数据
fn remove_value_attribute_field(_key_str: &str, _value: Value) {}