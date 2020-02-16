use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use json5;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Database {
    pub basic_data: BasicData,
    pub api_docs: HashMap<String, ApiDoc>,
    // {fileanme:api_doc}
    pub api_data: HashMap<String, Vec<Arc<Mutex<ApiData>>>>,
    // {url:[a_api_doc1, a_api_data2]}
    pub fileindex_data: HashMap<String, HashSet<String>>,
    // ref和相关文件的索引，当文件更新后，要找到所有ref他的地方，然后进行更新
    pub websocket_api: Arc<Mutex<ApiData>>,
    pub auth_doc: Option<AuthDoc>,
    pub settings: Option<Value>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiData {
    pub name: String,
    pub desc: String,
    pub url: String,
    pub url_param: Value,
    pub method: Vec<String>,
    pub auth: bool,
    pub body_mode: String,
    pub body: Value,
    pub query: Value,
    pub request_headers: Value,
    pub response_headers: Value,
    pub response: Value,
    pub test_data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// auth认证中心文档
pub struct AuthDoc {
    pub name: String,
    // auth 文档名称
    pub desc: String,
    // auth 相关说明
    pub auth_type: String,
    // auth 类型
    pub auth_place: String,
    // auth 放在什么地方：headers 或者是 url上
    pub filename: String,
    // 文件名称
    pub groups: Vec<AuthData>,
    pub no_perm_response: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthData {
    pub name: String,
    pub desc: String,
    pub users: HashMap<String, Value>,
    pub has_perms: HashMap<String, HashSet<String>>,
    pub no_perms: HashMap<String, HashSet<String>>,
    pub no_perm_response: Value,
}

fn fix_json(org_string: String) -> String {
    let re = Regex::new(r#":\s*"[\s\S]*?\n*[\s\S]*?""#).unwrap(); // 把多换行变为一个
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

/// 加载auth认证的相关数据
pub fn load_auth_data(api_docs: &HashMap<String, ApiDoc>) -> Option<AuthDoc> {
    let auth_files = ["_auth.json5", "_auth.json"];

    let mut auth_value = json!({});
    let mut filename = "";
    for file in auth_files.iter() {
        match fs::read_to_string(file) {
            Ok(v) => {
                let v = fix_json(v);
                match json5::from_str(&v) {
                    Ok(v) => {
                        filename = file;
                        auth_value = v;
                        break;
                    }
                    Err(e) => {
                        println!("Parse json file {} error : {:?}", file, e);
                        return None;
                    }
                }
            }
            Err(_) => return None,
        };
    }

    if filename == "" {
        return None;
    }

    let obj = auth_value.as_object().unwrap();

    let name = match obj.get("name") {
        Some(name) => name.as_str().unwrap(),
        None => "Panda api auth",
    };

    let desc = match obj.get("desc") {
        Some(name) => name.as_str().unwrap(),
        None => "Panda api desc",
    };

    let auth_type = match obj.get("auth_type") {
        Some(name) => name.as_str().unwrap(),
        None => "Bearer",
    };

    let auth_place = match obj.get("auth_place") {
        Some(v) => v.as_str().unwrap(),
        None => "headers",
    };

    let no_perm_response = match obj.get("no_perm_response") {
        Some(v) => v.clone(),
        None => json!({"code":-1, "error":"no perm to visit"}),
    };

    let mut groups: Vec<AuthData> = Vec::new();

    if let Some(test_data_value) = obj.get("groups") {
        if let Some(items) = test_data_value.as_array() {
            for data in items {
                let test_data_name = match data.get("name") {
                    Some(v) => v.as_str().unwrap(),
                    None => "",
                };
                let test_data_desc = match data.get("desc") {
                    Some(v) => v.as_str().unwrap(),
                    None => "",
                };

                let mut users: HashMap<String, Value> = HashMap::new();

                if let Some(v) = data.get("users") {
                    if let Some(uu) = v.as_array() {
                        for user in uu {
                            if let Some(t) = user.get("token") {
                                if let Some(token) = t.as_str() {
                                    users.insert(token.to_string(), user.clone());
                                }
                            }
                        }
                    }
                };

                let has_perms = parse_auth_perms(data.get("has_perms"), api_docs);
                let no_perms = parse_auth_perms(data.get("no_perms"), api_docs);

                let test_data_no_perm_response = match data.get("no_perm_response") {
                    Some(v) => v.clone(),
                    None => no_perm_response.clone(),
                };

                groups.push(AuthData {
                    name: test_data_name.to_string(),
                    desc: test_data_desc.to_string(),
                    users: users,
                    has_perms: has_perms,
                    no_perms: no_perms,
                    no_perm_response: test_data_no_perm_response,
                })
            }
        }
    }

    Some(AuthDoc {
        name: name.to_string(),
        desc: desc.to_string(),
        auth_type: auth_type.to_string(),
        auth_place: auth_place.to_string(),
        filename: filename.to_string(),
        groups: groups,
        no_perm_response: no_perm_response,
    })
}

pub fn load_basic_data() -> (BasicData, Option<Value>) {
    let settings_files = ["_settings.json5", "_settings.json"];

    let mut setting_value = json!({});
    let mut return_value: Option<Value> = None;
    for settings_file in settings_files.iter() {
        match fs::read_to_string(settings_file) {
            Ok(v) => {
                let v = fix_json(v);
                match json5::from_str(&v) {
                    Ok(v) => {
                        setting_value = v;
                        return_value = Some(setting_value.clone());
                        break;
                    }
                    Err(e) => {
                        println!("Parse json file {} error : {:?}", settings_file, e);
                    }
                }
            }
            Err(_) => (),
        };
    }

    let obj = setting_value.as_object().unwrap();

    let project_name = match obj.get("project_name") {
        Some(name) => name.as_str().unwrap(),
        None => "Panda api docs",
    };
    let project_name = project_name.to_string();

    let project_desc = match obj.get("project_desc") {
        Some(name) => name.as_str().unwrap(),
        None => "",
    };
    let project_desc = project_desc.to_string();

    let read_me = match fs::read_to_string("README.md") {
        Ok(x) => x,
        Err(_) => {
            if &project_desc == "" {
                "Panda api docs".to_string()
            } else {
                project_desc.clone()
            }
        }
    };

    let global_value = match obj.get("global") {
        Some(v) => v.clone(),
        None => Value::Null,
    };

    (
        BasicData {
            read_me,
            project_name,
            project_desc,
            global_value,
        },
        return_value,
    )
}

impl Database {
    /// 加载api docs 接口的json数据、配置、相关文档
    pub fn load() -> Database {
        let (basic_data, settings) = load_basic_data();

        let mut api_docs = HashMap::new();
        let mut api_data: HashMap<String, Vec<Arc<Mutex<ApiData>>>> = HashMap::new();
        let mut fileindex_data: HashMap<String, HashSet<String>> = HashMap::new();

        let websocket_api = Arc::new(Mutex::new(ApiData::default()));

        for entry in WalkDir::new("./") {
            let e = entry.unwrap();
            let doc_file = e.path().to_str().unwrap().trim_start_matches("./");
            Self::load_a_api_json_file(
                doc_file,
                &basic_data,
                &mut api_data,
                &mut api_docs,
                websocket_api.clone(),
                &mut fileindex_data,
            );
        }

        let auth_doc = load_auth_data(&api_docs);
        Database {
            basic_data,
            api_data,
            api_docs,
            fileindex_data,
            websocket_api,
            auth_doc,
            settings,
        }
    }

    /// 只加载一个api_doc文件的数据
    ///
    pub fn load_a_api_json_file(
        doc_file: &str,
        basic_data: &BasicData,
        api_data: &mut HashMap<String, Vec<Arc<Mutex<ApiData>>>>,
        api_docs: &mut HashMap<String, ApiDoc>,
        websocket_api: Arc<Mutex<ApiData>>,
        fileindex_data: &mut HashMap<String, HashSet<String>>,
    ) -> i32 {
        if !(doc_file.ends_with(".json") || doc_file.ends_with(".json5"))
            || doc_file == "_settings.json"
            || doc_file == "_settings.json5"
            || doc_file == "_auth.json"
            || doc_file == "_auth.json5"
            || doc_file.contains("_data/")
            || doc_file.starts_with(".")
            || doc_file.contains("/.")
        {
            return -1;
        }

        let d = match fs::read_to_string(doc_file) {
            Ok(d) => d,
            Err(_e) => {
                // println!("Unable to read file: {} {:?}", doc_file, e);
                // 文件被删除
                return -2;
            }
        };

        let d = fix_json(d);
        let json_value: Value = match json5::from_str(&d) {
            Ok(v) => v,
            Err(e) => {
                println!("Parse json file {} error : {:?}", doc_file, e);
                return -3;
            }
        };

        let doc_file_obj = match json_value.as_object() {
            Some(doc_file_obj) => doc_file_obj,
            None => return -4,
        };

        let mut doc_name = match doc_file_obj.get("name") {
            Some(name) => match name.as_str() {
                Some(v) => v.to_string(),
                None => format!("{}", name),
            },
            None => doc_file.to_string(),
        };
        if &doc_name == "" {
            // 如果接口文档name为空，那么就用文件名作为文档名称
            doc_name = doc_file.to_string();
        }

        let doc_desc = match doc_file_obj.get("desc") {
            Some(desc) => desc.as_str().unwrap(),
            None => "",
        };
        let doc_desc = doc_desc.to_string();

        let doc_order: i64 = match doc_file_obj.get("order") {
            Some(order) => order.as_i64().expect("order is not number"),
            None => 0,
        };

        let apis = match doc_file_obj.get("apis") {
            Some(api) => api.clone(),
            None => json!([]),
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
                    None => (),
                }

                let name = get_api_field_string_value(
                    "name",
                    doc_file.to_string(),
                    api,
                    &ref_data,
                    &basic_data.global_value,
                );
                let desc = get_api_field_string_value(
                    "desc",
                    "".to_string(),
                    api,
                    &ref_data,
                    &basic_data.global_value,
                );
                let url = get_api_field_string_value(
                    "url",
                    "".to_string(),
                    api,
                    &ref_data,
                    &basic_data.global_value,
                );

                let mut method = get_api_field_array_value(
                    "method",
                    vec!["GET".to_string()],
                    api,
                    &ref_data,
                    &basic_data.global_value,
                );
                for m in method.iter_mut() {
                    *m = m.to_uppercase();
                }

                let body_mode = get_api_field_string_value(
                    "body_mode",
                    "json".to_string(),
                    api,
                    &ref_data,
                    &basic_data.global_value,
                );
                let auth = get_api_field_bool_value(
                    "auth",
                    false,
                    api,
                    &ref_data,
                    &basic_data.global_value,
                );

                let url_param = match api.get("url_param") {
                    Some(url_param) => url_param.clone(),
                    None => match ref_data.get("url_param") {
                        Some(v) => v.clone(),
                        None => Value::Null,
                    },
                };
                let (mut ref_files, url_param) =
                    parse_attribute_ref_value(url_param, doc_file_obj, doc_file);

                let body = match api.get("body") {
                    Some(body) => body.clone(),
                    None => match ref_data.get("body") {
                        Some(v) => v.clone(),
                        None => Value::Null,
                    },
                };
                let (mut ref_files2, body) =
                    parse_attribute_ref_value(body, doc_file_obj, doc_file);
                ref_files.append(&mut ref_files2);

                let request_headers = match api.get("request_headers") {
                    Some(request_headers) => request_headers.clone(),
                    None => match ref_data.get("request_headers") {
                        Some(v) => v.clone(),
                        None => Value::Null,
                    },
                };
                let (mut ref_files2, request_headers) =
                    parse_attribute_ref_value(request_headers, doc_file_obj, doc_file);
                ref_files.append(&mut ref_files2);

                let response_headers = match api.get("response_headers") {
                    Some(response_headers) => response_headers.clone(),
                    None => match ref_data.get("response_headers") {
                        Some(v) => v.clone(),
                        None => Value::Null,
                    },
                };
                let (mut ref_files2, response_headers) =
                    parse_attribute_ref_value(response_headers, doc_file_obj, doc_file);
                ref_files.append(&mut ref_files2);

                let query = match api.get("query") {
                    Some(query) => query.clone(),
                    None => match ref_data.get("query") {
                        Some(v) => v.clone(),
                        None => Value::Null,
                    },
                };
                let (mut ref_files2, query) =
                    parse_attribute_ref_value(query, doc_file_obj, doc_file);
                ref_files.append(&mut ref_files2);

                // 最后查询global_value
                let mut response: Map<String, Value> =
                    match basic_data.global_value.pointer("/api/response") {
                        Some(v) => v.as_object().unwrap().clone(),
                        None => json!({}).as_object().unwrap().clone(),
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
                let (mut ref_files2, response) =
                    parse_attribute_ref_value(Value::Object(response), doc_file_obj, doc_file);

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
                    Some(test_data) => test_data.clone(),
                    None => match ref_data.get("test_data") {
                        Some(v) => v.clone(),
                        None => Value::Null,
                    },
                };

                let o_api_data = ApiData {
                    name,
                    desc,
                    body_mode,
                    body,
                    query,
                    response,
                    test_data,
                    url_param,
                    request_headers,
                    response_headers,
                    auth: auth,
                    url: url.clone(),
                    method: method.clone(),
                };
                let a_api_data = Arc::new(Mutex::new(o_api_data.clone()));

                if method.contains(&"WEBSOCKET".to_string()) {
                    let mut websocket_api = websocket_api.lock().unwrap();
                    *websocket_api = o_api_data.clone();
                }
                // 形成 { url: {method:api} }
                match api_data.get_mut(&url) {
                    Some(data) => {
                        data.push(a_api_data.clone());
                    }
                    None => {
                        let mut x = Vec::new();
                        x.push(a_api_data.clone());
                        api_data.insert(url.clone(), x);
                    }
                }
                api_vec.push(a_api_data.clone());
            }
        }

        let api_doc = ApiDoc {
            name: doc_name,
            desc: doc_desc,
            order: doc_order,
            filename: doc_file.to_string(),
            apis: api_vec,
        };
        api_docs.insert(doc_file.to_string(), api_doc);

        1
    }
}

fn load_ref_file_data(ref_file: &str, doc_file: &str) -> (String, Option<Value>) {
    let ref_info: Vec<&str> = ref_file.split(":").collect();

    match ref_info.get(0) {
        Some(filename) => {
            let mut file_path;
            if filename.starts_with("./_data") {
                let path = Path::new(doc_file).parent().unwrap();
                file_path = format!(
                    "{}/{}",
                    path.to_str().unwrap(),
                    filename.trim_start_matches("./")
                );
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
        None => (),
    };
    ("".to_string(), None)
}

/// 从value中获取array
fn get_array_from_value(key: &str, value: &Value) -> Option<Vec<String>> {
    if let Some(v) = value.get(key) {
        if v.is_string() {
            if let Some(v) = v.as_str() {
                return Some(vec![v.to_string()]);
            } else {
                return Some(vec![format!("{}", v)]);
            }
        } else if v.is_array() {
            if let Some(v_list) = v.as_array() {
                let mut r = Vec::new();
                for i in v_list {
                    if let Some(x) = i.as_str() {
                        r.push(x.to_string());
                    }
                }
                return Some(r);
            }
        }
    }
    None
}

/// 获取值可能是数组的字段值
/// 例如method，可能填写是字符串，也可能是数组
fn get_api_field_array_value(
    key: &str,
    default_value: Vec<String>,
    api: &Value,
    ref_data: &Value,
    global_data: &Value,
) -> Vec<String> {
    // 如果直接在api接口上有设置值
    if let Some(v) = get_array_from_value(key, api) {
        return v;
    }

    // 如果在ref_data上有设置值
    if let Some(v) = get_array_from_value(key, ref_data) {
        return v;
    }

    // 最后查询global_value
    if let Some(v) = global_data.get("apis") {
        if let Some(v) = get_array_from_value(key, v) {
            return v;
        }
    }
    default_value
}

/// 获取api里面字段的数据
/// 如 url, name等
fn get_api_field_string_value(
    key: &str,
    default_value: String,
    api: &Value,
    ref_data: &Value,
    global_data: &Value,
) -> String {
    match api.get(key) {
        Some(d) => {
            if let Some(v) = d.as_str() {
                return v.to_owned();
            } else {
                return format!("{}", d);
            }
        }
        None => (),
    }

    if let Some(d) = ref_data.get(key) {
        if let Some(v) = d.as_str() {
            return v.to_owned();
        } else {
            return format!("{}", d);
        }
    }

    // 最后查询global_value
    match global_data.get("apis") {
        Some(v) => match v.get(key) {
            Some(v2) => {
                if let Some(d) = v2.as_str() {
                    return d.to_owned();
                } else {
                    return format!("{}", v2);
                }
            }
            None => (),
        },
        None => (),
    }
    default_value
}

fn get_api_field_bool_value(
    key: &str,
    default_value: bool,
    api: &Value,
    ref_data: &Value,
    global_data: &Value,
) -> bool {
    match api.get(key) {
        Some(d) => {
            if let Some(v) = d.as_bool() {
                return v;
            } else {
                println!("{} value is not a bool", key)
            }
        }
        None => (),
    }

    if let Some(d) = ref_data.get(key) {
        if let Some(v) = d.as_bool() {
            return v;
        } else {
            println!("{} value is not a bool", key)
        }
    }

    match global_data.get("apis") {
        Some(v) => match v.get(key) {
            Some(d) => {
                if let Some(v2) = d.as_bool() {
                    return v2;
                } else {
                    println!("{} value is not a bool", key)
                }
            }
            None => (),
        },
        None => (),
    }

    default_value
}

/// parse $ref引用数据
/// 第一个参数表示获取到的值，body, query, resonse 等, 判断是否有引用值 或者 全局值
/// 对不满足要求的数据会全部进行过滤
fn parse_attribute_ref_value(
    value: Value,
    doc_file_obj: &Map<String, Value>,
    doc_file: &str,
) -> (Vec<String>, Value) {
    let mut ref_files: Vec<String> = Vec::new();
    if value.is_null() {
        return (ref_files, value);
    }

    if value.is_object() {
        let value_obj = value.as_object().unwrap();
        let mut new_value = value_obj.clone();

        // 处理文件引入
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
                                        new_v_str = format!(
                                            "{}{}",
                                            v3.as_str().unwrap(),
                                            &v_str[m.end()..]
                                        );
                                    }
                                    None => (),
                                }
                            }
                            None => (),
                        };
                    }
                    None => (),
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

            // 移除exclude中的字段
            if let Some(e) = value_obj.get("$exclude") {
                for v2 in e.as_array().unwrap() {
                    let key_str = v2.as_str().unwrap();
                    if key_str.contains("/") {
                        // 如果exclude中含有/斜杠，表示要嵌套的去移除字段
                        let v = remove_val_from_value(Value::Object(new_value), key_str);
                        new_value = v.as_object().unwrap().clone();
                    } else {
                        new_value.remove(key_str);
                    }
                }
            }
        }

        for (field_key, field_attrs) in value_obj {
            if field_attrs.is_string() && field_attrs.as_str().unwrap() == "$del" {
                new_value.remove(field_key);
                continue;
            } else if field_key == "$del"
                || field_key == "$ref"
                || field_key == "$exclude"
                || field_key == "$include"
            {
                continue;
            } else if field_key == "enum" || field_key == "$enum" {
                // --- start 处理 enum
                // 处理enum, enum不是数组，也不是对象，那么就会忽略这个字段
                let (mut ref_files2, field_value) =
                    parse_attribute_ref_value(field_attrs.clone(), doc_file_obj, doc_file);
                ref_files.append(&mut ref_files2);

                // 判断第一个是不是$desc, 或$符号开头，如果是，那么表示里面的就是[value, desc]组成的元素
                let mut is_desc_format = false;
                if field_key == "$enum" {
                    new_value.remove(field_key);
                    is_desc_format = true;
                }

                if let Some(enum_array) = field_value.as_array() {
                    let mut new_enum: Vec<Value> = Vec::new();

                    if enum_array.len() == 0 {
                        // 如果数组长度为空，那么忽略这个enum
                        new_value.remove(field_key);
                        continue;
                    }

                    for enum_item in enum_array {
                        // 取出枚举的值，重组为[{$value:1,$desc:""}]的形式
                        let mut new_item = Map::new();
                        if is_desc_format || enum_item.is_array() {
                            if is_desc_format {
                                // 如果是有注释模式
                                if let Some(enum_item2) = enum_item.as_array() {
                                    if enum_item2.len() < 1 {
                                        println!("doc file {} field {} enum value length error {:?}, $desc need all item is array: [[value1, desc2], [value2, desc2]...]", doc_file, field_key, enum_item);
                                        continue;
                                    }
                                    for (i, v) in enum_item2.iter().enumerate() {
                                        if i == 0 {
                                            new_item.insert("$value".to_string(), v.clone());
                                        } else if i == 1 {
                                            new_item.insert("$desc".to_string(), v.clone());
                                        }
                                    }
                                    if new_item.len() > 0 {
                                        new_enum.push(Value::Object(new_item));
                                    }
                                    continue;
                                }
                                // 如果不是数组，忽略
                                continue;
                            }
                        }

                        if enum_item.is_object() {
                            if let Some(enum_item2) = enum_item.as_object() {
                                if let Some(_v) = enum_item2.get("$value") {
                                    continue;
                                }
                            }
                        }
                        new_item.insert("$value".to_string(), enum_item.clone());
                        new_enum.push(Value::Object(new_item));
                    }
                    if new_enum.len() > 1 {
                        new_value.insert(
                            field_key.trim_start_matches("$").to_string(),
                            Value::Array(new_enum),
                        );
                    }
                } else {
                    // 如果不是数组，那么直接删除 enum属性
                    new_value.remove(field_key);
                    continue;
                }
                // --- end 处理 enum

                //                return (ref_files, Value::Object(new_value));
                break;
            }

            if let Some(is_del) = field_attrs.pointer("/$del") {
                // 处理当字段设置了{$del:true}属性,那么就不显示这个字段
                if let Some(true) = is_del.as_bool() {
                    new_value.remove(field_key);
                    continue;
                }
            }

            let (mut ref_files2, field_value) =
                parse_attribute_ref_value(field_attrs.clone(), doc_file_obj, doc_file);
            ref_files.append(&mut ref_files2);
            new_value.insert(field_key.trim_start_matches("$").to_string(), field_value);
        }

        // 处理嵌套增加或修改属性值的问题 category/category_name
        let mut new_value_value = Value::Object(new_value.clone());
        for (field_key, field_attrs) in &new_value {
            if field_key.contains("/") {
                new_value_value = modify_val_from_value(new_value_value, field_key, field_attrs);
            }
        }

        return (ref_files, new_value_value);
    } else if value.is_array() {
        // 处理array
        if let Some(value_array) = value.as_array() {
            if value_array.len() == 1 {
                if let Some(value_array_one) = value_array.get(0) {
                    let (ref_files, array_item_value) =
                        parse_attribute_ref_value(value_array_one.clone(), doc_file_obj, doc_file);
                    return (ref_files, Value::Array(vec![array_item_value]));
                } else {
                    println!(" file array value empty '{}' got {:?}", doc_file, value);
                }
            } else {
                return (ref_files, value);
            }
        }
    }

    (ref_files, value)
}

/// auth文件里面，可能是按文件加载接口地址
fn load_all_api_docs_url(
    result: &mut HashMap<String, HashSet<String>>,
    doc_file: &str,
    methods: HashSet<String>,
    api_docs: &HashMap<String, ApiDoc>,
    exclude: &HashMap<String, HashSet<String>>,
) {
    let mut all_methods: HashSet<String> = HashSet::with_capacity(7);
    for v in &["POST", "GET", "PUT", "DELETE", "OPTIONS", "HEAD", "PATCH"] {
        all_methods.insert(v.to_string());
    }

    let doc_file = doc_file.trim_start_matches("$");
    if let Some(api_doc) = api_docs.get(doc_file) {
        for a in &api_doc.apis {
            let api = a.lock().unwrap();
            // 如果exclude 排除这个url，并且排除所有方法，那么就没有任何这个url的权限
            if let Some(exclude_methods) = exclude.get(&api.url) {
                if exclude_methods.is_empty() {
                    result.insert(api.url.clone(), methods.clone());
                    continue;
                }

                if exclude_methods.contains("*") {
                    continue;
                }

                let mut new_methods: HashSet<String>;
                if methods.contains("*") {
                    new_methods = all_methods.clone();
                } else {
                    new_methods = methods.clone();
                }

                for m in exclude_methods {
                    new_methods.remove(&m.to_uppercase());
                }

                if new_methods.len() > 0 {
                    if new_methods == all_methods {
                        let mut m = HashSet::new();
                        m.insert("*".to_string());
                        result.insert(api.url.clone(), m);
                    } else {
                        result.insert(api.url.clone(), new_methods);
                    }
                }
            } else {
                result.insert(api.url.clone(), methods.clone());
            }
        }
    }
}

/// 把权限解析为一个map
fn parse_auth_perms(
    perms_data: Option<&Value>,
    api_docs: &HashMap<String, ApiDoc>,
) -> HashMap<String, HashSet<String>> {
    let mut result: HashMap<String, HashSet<String>> = HashMap::new();
    if let Some(perms) = perms_data {
        if let Some(perms) = perms.as_array() {
            for perm in perms {
                let mut methods = HashSet::new();
                let mut url = "";
                let mut exclude: HashMap<String, HashSet<String>> = HashMap::new();

                match perm {
                    Value::String(perm_str) => {
                        // 如果直接是一个字符串，表示字符串就是接口，然后拥有所有请求方法
                        methods.insert("*".to_string());
                        url = perm_str;
                    }
                    Value::Array(perm_array) => {
                        for (i, p) in perm_array.iter().enumerate() {
                            match p {
                                Value::String(perm_str) => {
                                    if i == 0 {
                                        url = perm_str;
                                    } else {
                                        methods.insert(perm_str.to_uppercase());
                                    }
                                }
                                _ => continue,
                            }
                        }
                    }
                    Value::Object(perm_obj) => {
                        exclude = parse_auth_perms(perm_obj.get("$exclude"), api_docs);
                        if let Some(m) = perm_obj.get("methods") {
                            if m.is_string() {
                                let m = m.as_str().unwrap();
                                methods.insert(m.to_uppercase());
                            } else if m.is_array() {
                                let m = m.as_array().unwrap();
                                for i in m {
                                    let i = i.as_str().unwrap();
                                    methods.insert(i.to_uppercase());
                                }
                            }
                        } else {
                            methods.insert("*".to_string());
                        }

                        // 如果是一个对象，那么可能是{$ref:"auth.json5", $exclude:["/login/", ["/logout/", "GET", "POST"]]}
                        if let Some(perm_str) = perm_obj.get("$ref") {
                            if let Some(perm_str) = perm_str.as_str() {
                                load_all_api_docs_url(
                                    &mut result,
                                    perm_str,
                                    methods,
                                    api_docs,
                                    &exclude,
                                );
                            }

                            continue;
                        }

                        url = match perm_obj.get("url") {
                            Some(url) => url.as_str().unwrap(),
                            None => continue,
                        };
                    }
                    _ => {
                        continue;
                    }
                }

                // 如果没有设置methods，默认就是所有方法
                if url.starts_with("$") {
                    // 按接口文件加载urls
                    load_all_api_docs_url(&mut result, url, methods, api_docs, &exclude);
                } else {
                    result.insert(url.to_string(), methods);
                }
            }
        } else if perms.is_string() {
            let url = perms.as_str().unwrap();
            let mut methods = HashSet::new();
            methods.insert("*".to_string());
            if url.starts_with("$") {
                // 按接口文件加载urls
                let exclude: HashMap<String, HashSet<String>> = HashMap::new();
                load_all_api_docs_url(&mut result, url, methods, api_docs, &exclude);
            } else {
                result.insert(url.to_string(), methods);
            }
        }
    };
    result
}

fn parse_index(s: &str) -> Option<usize> {
    if s.starts_with('+') || (s.starts_with('0') && s.len() != 1) {
        return None;
    }
    s.parse().ok()
}

/// 为修改value中嵌套的某一个值或者增加某一个值 /category/id
fn modify_val_from_value(mut value: Value, pointer: &str, new_value: &Value) -> Value {
    if pointer == "" {
        return value;
    }

    let tokens: Vec<&str> = pointer
        .trim_start_matches("/")
        .trim_end_matches("/")
        .split("/")
        .collect();
    if tokens.len() == 0 {
        return value;
    }
    let mut target = &mut value;
    let l = tokens.len() - 1;

    for (i, &token) in tokens.iter().enumerate() {
        let target_once = target;
        let target_opt = match target_once {
            Value::Object(ref mut map) => {
                if i == 0 {
                    map.remove(pointer);
                }
                if i == l {
                    map.insert(token.to_string(), new_value.clone());
                    break;
                } else {
                    map.get_mut(token)
                }
            }
            Value::Array(ref mut list) => parse_index(&token).and_then(move |x| list.get_mut(x)),
            _ => break,
        };

        if let Some(t) = target_opt {
            target = t;
        } else {
            break;
        }
    }

    value
}

/// 从value中嵌套的删除某一个值 /category/id
fn remove_val_from_value(mut value: Value, pointer: &str) -> Value {
    if pointer == "" {
        return value;
    }

    let tokens: Vec<&str> = pointer
        .trim_start_matches("/")
        .trim_end_matches("/")
        .split("/")
        .collect();
    if tokens.len() == 0 {
        return value;
    }
    let mut target = &mut value;
    let l = tokens.len() - 1;
    for (i, &token) in tokens.iter().enumerate() {
        let target_once = target;
        let target_opt = match target_once {
            Value::Object(ref mut map) => {
                if i == l {
                    map.remove(token);
                    break;
                } else {
                    map.get_mut(token)
                }
            }
            Value::Array(ref mut list) => parse_index(&token).and_then(move |x| list.get_mut(x)),
            _ => break,
        };

        if let Some(t) = target_opt {
            target = t;
        } else {
            break;
        }
    }

    value
}
