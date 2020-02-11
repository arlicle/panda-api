use actix_web::{http, web, Error, HttpRequest, HttpResponse};
use actix_web::dev::ResourceDef;
use std::time::{Duration, Instant, SystemTime};
use std::collections::{HashMap, HashSet};
use actix_files;

use rand::{thread_rng, Rng};

use actix_multipart::Multipart;
use futures::StreamExt;
use regex::Regex;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value, Map};
use std::fs;
use std::io::prelude::*;
use std::sync::Mutex;
use actix_web_actors::ws;

use crate::db;
use crate::websocket::WsChatSession;
use crate::server;
use actix::*;
use crate::mock;
use crate::{int, float, timestamp};


#[derive(Serialize, Deserialize, Debug)]
pub struct ApiDocDataRequest {
    filename: String,
}


/// 根据接口文件路径获取接口文档详情
pub async fn get_api_doc_data(req_get: web::Query<ApiDocDataRequest>, data: web::Data<Mutex<db::Database>>) -> HttpResponse {
    let data = data.lock().unwrap();
    let api_docs = &data.api_docs;

    if req_get.filename == "_auth.json5" {
        if let Some(auth_doc) = &data.auth_doc {
            return HttpResponse::Ok().json(auth_doc);
        } else {
            return HttpResponse::Ok().body("");
        }
    } else if req_get.filename == "_settings.json5" {
        if let Some(settings) = &data.settings {
            return HttpResponse::Ok().json(settings);
        } else {
            return HttpResponse::Ok().body("");
        }
    }

    for (_, doc) in api_docs {
        if doc.filename == req_get.filename {
            let mut apis = Vec::new();
            for api in &doc.apis {
                let api = api.lock().unwrap();
                apis.push({ &*api }.clone());
            }
            return HttpResponse::Ok().json(
                json!({
                    "name": doc.name,
                    "desc": doc.desc,
                    "order": doc.order,
                    "filename": doc.filename,
                    "apis": apis}));
        }
    }

    HttpResponse::Ok().json(json!({
      "code": -1,
      "msg": "没有该接口文档文件"
    }))
}


/// 获取项目接口的基本信息
/// 返回项目名称，介绍，项目接口简要列表
/// 前端需要自己根据 api_doc 的order进行排序
pub async fn get_api_doc_basic(data: web::Data<Mutex<db::Database>>) -> HttpResponse {
    let data = data.lock().unwrap();
    let basic_data = &data.basic_data;
    let api_docs = &data.api_docs;

    let mut docs = Vec::new();
    if let Some(auth_doc) = &data.auth_doc {
        docs.push(json!({"name":auth_doc.name, "desc":auth_doc.desc, "order":0, "filename":"_auth.json5"}));
    }
    if let Some(_) = &data.settings {
        docs.push(json!({"name":"Settings", "desc":"", "order":0, "filename":"_settings.json5"}));
    }
    for (_, doc) in api_docs {
        docs.push(json!({ "name": doc.name, "desc": doc.desc, "order": doc.order, "filename": doc.filename }));
    }

    HttpResponse::Ok().json(json!({
      "project_name": &basic_data.project_name,
      "project_desc": &basic_data.project_desc,
      "read_me": &basic_data.read_me,
      "api_docs": docs
    }))
}


/// api docs 在线浏览文档
/// 前端相关静态皮肤文件展示服务
pub async fn theme_view(req: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    let req_path = req.path();

    // for api documents homepage
    let home_dir = dirs::home_dir().unwrap();
    let theme_home_dir = format!("{}/.panda_api/theme", home_dir.to_str().unwrap().trim_end_matches("/"));
    let theme_file;
    if req_path == "/" {
        theme_file = "/index.html";
    } else if req_path == "favicon.ico" {
        theme_file = "/static/favicon.ico";
    } else {
        theme_file = req_path;
    }

    let theme_filepath = format!("{}{}", theme_home_dir, theme_file);

    return Ok(actix_files::NamedFile::open(theme_filepath)?);
}

/// 查看上传的 图片或文件
pub async fn upload_file_view(req: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    let req_path = req.path();
    let mut file_path = "_data".to_string() +  req_path;
    return Ok(actix_files::NamedFile::open(file_path)?);
}



/// 获取_data目录中的数据, models数据 或者其它加载数据
pub async fn get_api_doc_schema_data(req_get: web::Query<ApiDocDataRequest>) -> HttpResponse {
    let read_me = match fs::read_to_string(&req_get.filename) {
        Ok(x) => x,
        Err(_) => "no data file".to_string()
    };

    HttpResponse::Ok().content_type("application/json").body(read_me)
}


#[derive(Deserialize, Debug)]
pub struct FormData {
    username: String,
}


pub async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<HttpResponse, Error> {
    println!("s {:?}", req);
    ws::start(
        WsChatSession {
            id: 0,
            hb: Instant::now(),
            room: "Main".to_owned(),
            name: None,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}


/// 处理post、put、delete 请求
///
pub async fn action_handle(req: HttpRequest, request_body: Option<web::Json<Value>>, request_query: Option<web::Query<Value>>, request_form_data: Option<Multipart>, db_data: web::Data<Mutex<db::Database>>) -> HttpResponse {
    let body_mode = get_request_body_mode(&req);
    let req_method = req.method().as_str();

    if req_method == "OPTIONS" {
        return HttpResponse::Ok().body("");
    }

    let form_data = if &body_mode == "form-data" {
        get_request_form_data(request_form_data).await
    } else {
        Value::Null
    };

    let request_body = match request_body {
        Some(x) => {
            x.into_inner()
        }
        None => Value::Null
    };

    let request_query = match request_query {
        Some(x) => x.into_inner(),
        None => Value::Null
    };

    find_response_data(&req, body_mode, request_body, request_query, form_data, db_data)
}


/// 找到对应url 对应请求的数据
///
fn find_response_data(req: &HttpRequest, body_mode: String, request_body: Value, request_query: Value, form_data: Value, db_data: web::Data<Mutex<db::Database>>) -> HttpResponse {
    let db_data = db_data.lock().unwrap();
    let db_api_data = &db_data.api_data;
    let req_path = req.path();
    let req_method = req.method().as_str();
    let req_headers = req.headers();


    let api_data_list = match db_api_data.get(req_path) {
        Some(v) => Some(v),
        None => {
            let mut r = None;
            for (api_url, api_data_list) in db_api_data {
                let res = ResourceDef::new(api_url);
                if res.is_match(req_path) {
                    r = Some(api_data_list);
                    break;
                }
            }
            r
        }
    };

    if let Some(api_data_list) = api_data_list {
        'a: for a_api_data in api_data_list {
            let a_api_data = a_api_data.lock().unwrap();
            if a_api_data.method.contains(&req_method.to_string()) || a_api_data.method.contains(&"*".to_string()) {
                if a_api_data.auth { // 权限检查
                    if let Some(auth_valid_errors) = auth_validator(&req, &a_api_data.url, &db_data.auth_doc) {
                        return HttpResponse::Ok().json(auth_valid_errors);
                    }
                }

                // 首先判断在api接口中是否对request_headers有要求，如果有要求，那么就按照api接口定义匹配
                // 像url, 级别的接口匹配
                // 在api层面匹配成功后，才会有test_data层级的匹配
                // request_headers的匹配规则是，只判断test_case中的request_headers字段中的值在request_headers中是否有, 如果有就表示通过
                if !a_api_data.request_headers.is_null() {
                    if let Some(api_headers) = a_api_data.request_headers.as_object() {
                        for (header_key, header_field) in api_headers {
                            let mut header_value = "";
                            if let Some(header_field) = header_field.as_object() {
                                if let Some(v) = header_field.get("value") {
                                    header_value = v.as_str().unwrap();
                                }
                            } else {
                                header_value = header_field.as_str().unwrap();
                            }

                            if let Some(v) = req_headers.get(header_key) {
                                if let Ok(v_str) = v.to_str() {
                                    if v_str != header_value {
                                        continue 'a;
                                    }
                                }
                            }
                        }
                    }
                }


                let mut status_code = 200;
                let mut content_type = "application/json";
                if !a_api_data.response_headers.is_null() {
                    if let Some(api_headers) = a_api_data.response_headers.as_object() {
                        if let Some(s) = api_headers.get("status_code") {
                            if s.is_u64() {
                                status_code = s.as_u64().unwrap();
                            } else if s.is_object() {
                                if let Some(s) = s.as_object() {
                                    if let Some(v) = s.get("value") {
                                        status_code = v.as_u64().unwrap();
                                    }
                                }
                            }
                        }

                        if let Some(s) = api_headers.get("content_type") {
                            if s.is_string() {
                                content_type = s.as_str().unwrap();
                            } else if s.is_object() {
                                if let Some(s) = s.as_object() {
                                    if let Some(v) = s.get("value") {
                                        content_type = v.as_str().unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
                if status_code < 100 || status_code >= 600 {
                    status_code = 200;
                }
                let status_code = http::StatusCode::from_u16(status_code as u16).unwrap();

                // 开始匹配 test_data
                if let Some(test_data) = a_api_data.test_data.as_array() {
                    for test_case_data in test_data {
                        // 如果在test_data中设置了url，那么就要进行url匹配，如果不设置就不进行

                        if let Some(url) = test_case_data.get("url") {
                            if url != req_path {
                                continue;
                            }
                        }


                        if let Some(method) = test_case_data.get("method") {
                            // method属于有就匹配，没有就不匹配
                            if !is_request_method_match_test_case(req_method, method) {
                                continue;
                            }
                        }

                        let v = match test_case_data.get("body") {
                            Some(v) => v,
                            None => &Value::Null
                        };
                        if !is_value_equal(&request_body, v) {
                            continue;
                        }

                        let v = match test_case_data.get("form-data") {
                            Some(v) => v,
                            None => &Value::Null
                        };
                        if !is_value_equal(&form_data, v) {
                            continue;
                        }

                        let v = match test_case_data.get("query") {
                            Some(v) => v,
                            None => &Value::Null
                        };
                        let request_query = parse_request_query_to_api_query_format(&request_query, &a_api_data.query);
                        if !is_value_equal(&request_query, v) {
                            continue;
                        }

                        let case_response = match test_case_data.get("response") {
                            Some(v) => v,
                            None => &Value::Null
                        };

                        let case_response = parse_test_case_response(case_response, "", &a_api_data.response);
                        let serialized = serde_json::to_string(&case_response).unwrap();
                        return HttpResponse::build(status_code).content_type(content_type).body(serialized);
//                      return HttpResponse::Ok().json(case_response);
                    }
                }

                let x = create_mock_value(&a_api_data.response);
                let serialized = serde_json::to_string(&x).unwrap();
                return HttpResponse::build(status_code).content_type(content_type).body(serialized);
//                return HttpResponse::Ok().json(x);
            }
        }
        return HttpResponse::Ok().json(json!({
            "code": - 1,
            "msg": format ! ("this api address {} not match method {}", req_path, req_method)
        }));
    }

    HttpResponse::Ok().json(json!({
        "code": - 1,
        "msg": format ! ("this api address {} no api url match", req_path)
    }))
}


/// 处理test_case response中的部分$mock字段
fn parse_test_case_response(test_case_response: &Value, field_path: &str, response_model: &Value) -> Value {
    if test_case_response.is_null() {
        return Value::Null;
    }
    let mut result = Map::new();
    if let Some(response) = test_case_response.as_object() {
        for (field_key, field) in response {
            match field {
                Value::Object(field_obj) => {
                    if let Some(v) = field_obj.get("$mock") {
                        if let Some(v2) = v.as_bool() {
                            if v2 == true {
                                let pointer = format!("{}/{}", field_path, field_key);
                                if let Some(model_field) = response_model.pointer(&pointer) {
                                    let mut new_model_field_attr: Map<String, Value> = Map::new();
                                    if let Some(model_field_obj) = model_field.as_object() {
                                        new_model_field_attr = model_field_obj.clone();
                                        for (k2, v2) in field_obj {
                                            if k2 == "$mock" {
                                                continue;
                                            }
                                            new_model_field_attr.insert(k2.to_string(), v2.clone());
                                        }
                                    }

                                    let mut m = Map::new();
                                    m.insert(field_key.to_string(), Value::Object(new_model_field_attr));
                                    let v = create_mock_value(&Value::Object(m));
                                    for (k, v2) in v {
                                        result.insert(k, v2);
                                    }
                                }
                            }
                        }
                    } else {
                        let pointer = format!("{}/{}", field_path, field_key);
                        let v = parse_test_case_response(field, &pointer, response_model);
                        result.insert(field_key.to_string(), v);
                    }
                },
                Value::Array(field_array) => {
                    if field_array.len() >= 1 {
                        let v = &field_array[0];
                        let pointer = format!("{}/{}/0", field_path, field_key);
                        let v = parse_test_case_response(v, &pointer, response_model);
                        result.insert(field_key.to_string(), v);
                    }
                },
                _ => {
                    result.insert(field_key.to_string(), field.clone());
                }
            }
        }
    }

    Value::Object(result)
}


/// 把request_query 转换为api query的格式
fn parse_request_query_to_api_query_format(request_query: &Value, api_query: &Value) -> Value {
    if api_query.is_null() {
        return request_query.clone();
    }

    if let Some(api_query_data) = api_query.as_object() {
        if let Some(request_query) = request_query.as_object() {
            let mut result: Map<String, Value> = Map::new();
            for (field_key, field_value) in api_query_data {
                if let Some(request_query_value) = request_query.get(field_key) {
                    if let Some(field_value) = field_value.as_object() {
                        let mut field_type = "string";
                        if let Some(f_type) = field_value.get("type") {
                            if let Some(f_type) = f_type.as_str() {
                                field_type = f_type;
                            }
                        }
                        let field_type = field_type.to_lowercase();
                        let field_type = field_type.as_str();

                        if let Some(request_query_value_str) = request_query_value.as_str() {
                            match field_type {
                                "number" | "int" | "posint" | "negint" | "timestamp" => {
                                    if let Ok(v) = request_query_value_str.parse::<i64>() {
                                        result.insert(field_key.to_string(), json!(v));
                                    }
                                }
                                "float" | "posfloat" | "negfloat" => {
                                    if let Ok(v) = request_query_value_str.parse::<f64>() {
                                        result.insert(field_key.to_string(), json!(v));
                                    }
                                }
                                _ => {
                                    result.insert(field_key.to_string(), json!(request_query_value_str));
                                }
                            }
                        }
                    }
                }
            }

            return Value::Object(result);
        }
    }

    request_query.clone()
}


fn is_request_method_match_test_case(request_method: &str, test_case_method: &Value) -> bool {
    if test_case_method.is_string() {
        if let Some(test_case_method) = test_case_method.as_str() {
            if test_case_method == request_method || test_case_method == "*" {
                return true;
            }
        }
    } else if test_case_method.is_array() {
        // 如果method是一个数组
        if let Some(method_list) = test_case_method.as_array() {
            for method in method_list {
                if let Some(test_case_method) = method.as_str() {
                    if test_case_method == request_method || test_case_method == "*" {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// 判断两个serde value的值是否相等
/// 只要value2中要求的每个字段，value1中都有，就表示相等, 也就是说value1的字段可能会比value2多
/// 改为两个value1，value2中的字段必须完全相等
fn is_value_equal(value1: &Value, value2: &Value) -> bool {
    if value1.is_null() && value2.is_null() {
        return true;
    }
    match value1 {
        Value::Object(value1_a) => {
            match value2.as_object() {
                Some(value2_a) => {
                    if value1_a.is_empty() && value2_a.is_empty() {
                        return true;
                    }
                    if value1_a.len() != value2_a.len() {
                        return false;
                    }
                    for (k, v) in value2_a {
                        match value1_a.get(k) {
                            // 判断请求数据 与测试数据集的每个字段的值是否相等
                            Some(v2) => {
                                if v2 != v {
                                    return false;
                                }
                            }
                            None => {
                                return false;
                            }
                        }
                    }

                    return true;
                }
                None => {
                    if value1_a.is_empty() && value2.is_null() {
                        return true;
                    }
                    return false;
                }
            }
        }
        Value::Array(value1_array) => {
            match value2.as_array() {
                Some(value2_array) => {
                    if value1_array == value2_array {
                        return true;
                    }
                }
                None => {
                    if value1_array.is_empty() && value2.is_null() {
                        return true;
                    }
                    return false;
                }
            }
        }
        Value::Null => {
            // 让null 和 empty一样的相等
            match value2.as_object() {
                Some(value2_a) => {
                    if value2_a.is_empty() {
                        return true;
                    }
                }
                None => {
                    return false;
                }
            }
        }
        _ => {
            println!("Invalid Json Struct {:?}", value1);
        }
    }
    false
}


/// 从请求中获取form_data里面的数据以及文件上传
async fn get_request_form_data(request_form_data: Option<Multipart>) -> Value {
    let mut form_data: Map<String, Value> = Map::new();
    if let Some(mut payload) = request_form_data {
        // 如果是文件上传
        while let Some(item) = payload.next().await {
            if let Ok(mut field) = item {
                let content_type = match field.content_disposition() {
                    Some(v) => v,
                    None => {
                        break;
                    }
                };
                let x = field.headers().clone();
                let x = x.get("content-disposition").unwrap().to_str().unwrap();
                let re = Regex::new(r#"form-data; name="\w+""#).unwrap();

                let mut field_name = "";
                if let Some(m) = re.find(x) {
                    field_name = &x[m.start() + 17..m.end() - 1];
                };

                let mut filename = "";
                if let Some(f) = content_type.get_filename() {
                    filename = f;
                }

                match std::fs::create_dir_all("./_data/_upload") {
                    Ok(_) => (),
                    Err(e) => {
                        println!("create folder failed _data/_upload {:?}", e);
                    }
                }

                let filepath = format!("./_data/_upload/{}", filename);
                let filepath2 = &format!("./_data/_upload/{}", filename);

                if let Ok(mut f) = web::block(|| std::fs::File::create(filepath)).await {
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();

                        if let Ok(_) = f.write_all(&data) {
                            form_data.insert(field_name.to_string(), Value::String(filename.to_string()));
                            form_data.insert(format!("__{}", field_name), Value::String(format!("/_upload/{}", filename)));
                        } else {
                            println!("create file error {}", filepath2);
                        }
                    }
                } else {
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        let x = data.to_vec();
                        let v = std::str::from_utf8(&x).unwrap();
                        form_data.insert(field_name.to_string(), Value::String(v.to_string()));
                    }
                }
                continue;
            }
            break;
        }
    }
    Value::Object(form_data)
}


/// 获取请求的request_body
fn get_request_body_mode(req: &HttpRequest) -> String {
    let req_method = req.method().as_str();
    if req_method == "GET" {
        return "".to_string();
    }

    if let Some(head_value) = req.headers().get("content-type") {
        if let Ok(value_str) = head_value.to_str() {
            if value_str == "application/json" {
                return "json".to_string();
            } else if value_str.starts_with("multipart/form-data;") {
                return "form-data".to_string();
            } else if value_str == "text/plain" {
                return "text".to_string();
            } else if value_str == "application/javascript" {
                return "javascript".to_string();
            } else if value_str == "text/html" {
                return "html".to_string();
            } else if value_str == "application/xml" {
                return "xml".to_string();
            }
        }
    }

    "".to_string()
}

/// 判断是否是websocket连接请求
fn is_websocket_connect(req: &HttpRequest) -> bool {
    let mut has_version = false;
    let mut has_key = false;
    if let Some(_) = req.headers().get("sec-websocket-version") {
        has_version = true;
    }
    if let Some(_) = req.headers().get("sec-websocket-key") {
        has_key = true;
    }
    if has_version && has_key {
        return true;
    }
    false
}


fn get_token_from_request(req: &HttpRequest) -> String {
    let headers = req.headers();
    let mut token = "";
    if let Some(x) = headers.get("authorization") {
        if let Ok(x) = x.to_str() {
            token = x.trim_start_matches("Bearer ").trim();
        }
    }
    token.to_string()
}


/// 判断是否有某个url的权限
fn is_has_perm(url: &str, method: &str, perms: &HashMap<String, HashSet<String>>) -> bool {
    if let Some(methods) = perms.get("*") {
        // 如果有所有网址权限，再判断方法上的权限是否满足
        if methods.contains(method) || methods.contains("*") || methods.len() == 0 {
            return true;
        }
    }

    match perms.get(url) {
        Some(methods) => {
            // 如果有所有网址权限，再判断方法上的权限是否满足
            if methods.contains(method) || methods.contains("*") || methods.len() == 0 {
                return true;
            }
        }
        None => return false
    }

    false
}


/// 判断用户是否有当前接口访问权限，如果有权限返回None，如果没有权限 返回报错信息
fn auth_validator<'a>(req: &HttpRequest, api_url: &str, auth_doc: &'a Option<db::AuthDoc>) -> Option<&'a Value> {
    let token = get_token_from_request(req);

    if let Some(auth_data) = auth_doc {
        let no_perm_response = &auth_data.no_perm_response;

        // 判断token是否符合生成规则
        if &token == "" {
            return Some(no_perm_response);
        }

        let req_method = req.method().as_str();

        for group in &auth_data.groups {
            let mut group_no_perm_response = &group.no_perm_response;
            if group_no_perm_response.is_null() {
                group_no_perm_response = no_perm_response;
            }
            for (t, _) in &group.users {
                if t == &token {
                    // 判断请求是否在权限范围内
                    if is_has_perm(api_url, req_method, &group.no_perms) {
                        return Some(group_no_perm_response);
                    } else if is_has_perm(api_url, req_method, &group.has_perms) {
                        return None;
                    }
                    return Some(group_no_perm_response);
                }
            }
        }
        return Some(no_perm_response);
    }
    None
}


pub fn get_field_type(field_attr: &Value) -> String {
    let field_type = match field_attr.get("type") {
        Some(v) => v.as_str().unwrap(),
        None => {
//            if let Some(v) = field_attr.get("-type") {
//                v.as_str().unwrap()
//            } else
            if field_attr.is_array() {
                "array"
            } else if field_attr.is_object() {
                if let Some(field_attr_object) = field_attr.as_object() {
                    let mut s = "string";
                    for (_k, v) in field_attr_object {
                        if v.is_object() {
                            s = "object";
                            break;
                        }
                    }
                    s
                } else {
                    "string"
                }
            } else {
                "string"
            }
        }
    };

    field_type.to_lowercase()
}




macro_rules! get_string_value {
    ($field_key:expr, $field_type:ident, $field_attr:expr, $result:expr) => {
        let mut min_length = 0;
        let mut max_length = 0;
        let mut length = 0;
        let mut content_type = "markdown";

        if let Some(min_value1) = $field_attr.get("length") {
            if let Some(min_value1) = min_value1.as_u64() {
                length = min_value1;
            }
        }

        if let Some(min_value1) = $field_attr.get("min_length") {
            if let Some(min_value1) = min_value1.as_u64() {
                min_length = min_value1;
            }
        }

        if let Some(max_value1) = $field_attr.get("max_length") {
            if let Some(max_value1) = max_value1.as_u64() {
                max_length = max_value1;
            }
        }

        if let Some(min_value1) = $field_attr.get("content_type") {
            if let Some(min_value1) = min_value1.as_str() {
                content_type = min_value1;
            }
        }

        match $field_type {
            "cword"|"cw" => {
                $result.insert($field_key.clone(), Value::String(mock::text::cword(length as usize, min_length, max_length)));
            },
            "ctitle" | "ct" => {
                $result.insert($field_key.clone(), Value::String(mock::text::ctitle(length, min_length, max_length)));
            },
            "csentence" | "cs" => {
                $result.insert($field_key.clone(), Value::String(mock::text::csentence(length, min_length, max_length)));
            },
            "csummary" | "cm" => {
                $result.insert($field_key.clone(), Value::String(mock::text::csummary(length, min_length, max_length)));
            },
            "cparagraph" | "cp" => {
                $result.insert($field_key.clone(), Value::String(mock::text::cparagraph(length, min_length, max_length, content_type)));
            },
            "word" => {
                $result.insert($field_key.clone(), Value::String(mock::text::word(length as usize, min_length, max_length)));
            },
            "title" => {
                $result.insert($field_key.clone(), Value::String(mock::text::title(length, min_length, max_length)));
            },
            "sentence" => {
                $result.insert($field_key.clone(), Value::String(mock::text::sentence(length, min_length, max_length)));
            },
            "summary" => {
                $result.insert($field_key.clone(), Value::String(mock::text::summary(length, min_length, max_length)));
            },
            "paragraph" => {
                $result.insert($field_key.clone(), Value::String(mock::text::paragraph(length, min_length, max_length, content_type)));
            },
            "string" | _ => {
                $result.insert($field_key.clone(), Value::String(mock::basic::string(length, min_length, max_length)));
            }
        }
    }
}




/// 根据response定义生成返回给前端的mock数据
///
pub fn create_mock_value(response_model: &Value) -> Map<String, Value> {
    let mut result: Map<String, Value> = Map::new();
    if response_model.is_object() {
        let response_model = response_model.as_object().unwrap();
        let mut rng = thread_rng();

        for (field_key, field_attr) in response_model {
            if field_key == "$type" || field_key == "$name" || field_key == "$desc" || field_key == "$length" || field_key == "$min_length" || field_key == "$max_length" {
                continue;
            }

            let field_type = get_field_type(field_attr);
            let field_type = field_type.as_str();

            let mut required = true;

            match field_attr.get("required") {
                Some(v) => {
                    if let Some(v) = v.as_bool() {
                        required = v;
                    }
                }
                None => ()
            }

            if !required {
                // 如果required是false，那么返回数据就随机丢失
                let n = rng.gen_range(0, 10);
                if n % 2 == 0 {
                    continue;
                }
            }

            if let Some(value1) = field_attr.get("value") {
                // 如果设定了value，那么就只返回一个固定的值
                if let Some(value1) = value1.as_i64() {
                    result.insert(field_key.clone(), Value::from(value1));
                    continue;
                }
            }
            if let Some(enum_data) = field_attr.get("enum") {
                // 如果设置了枚举值，那么就只使用枚举值
                let list = enum_data.as_array().unwrap();
                if list.len() == 0 {
                    result.insert(field_key.clone(), Value::Null);
                    continue;
                }
                let n = rng.gen_range(0, list.len());
                let v = &list[n];
                match v {
                    Value::Object(v2) => {
                        if let Some(v3) = v2.get("$value") {
                            result.insert(field_key.clone(), v3.clone());
                        } else {
                            result.insert(field_key.clone(), v.clone());
                        }
                    }
                    _ => {
                        result.insert(field_key.clone(), v.clone());
                    }
                }

                continue;
            }

            match field_type {
                "float" | "posfloat" | "negfloat" => {
                    let mut min_value = i32::min_value() as f64;
                    let mut max_value = i32::max_value() as f64;

                    let mut decimal_places = 0;
                    let mut min_decimal_places = 0;
                    let mut max_decimal_places = 0;

                    match field_type {
                        "posfloat" => {
                            min_value = 0.0;
                        }
                        "negfloat" => {
                            max_value = 0.0;
                        }
                        _ => ()
                    }

                    if let Some(min_value1) = field_attr.get("min_value") {
                        if let Some(min_value1) = min_value1.as_f64() {
                            min_value = min_value1;
                        }
                    }

                    if let Some(max_value1) = field_attr.get("max_value") {
                        if let Some(max_value1) = max_value1.as_f64() {
                            max_value = max_value1;
                        }
                    }

                    if let Some(max_value1) = field_attr.get("min_decimal_places") {
                        if let Some(max_value1) = max_value1.as_u64() {
                            min_decimal_places = max_value1 as u32;
                        }
                    }
                    if let Some(max_value1) = field_attr.get("max_decimal_places") {
                        if let Some(max_value1) = max_value1.as_u64() {
                            max_decimal_places = max_value1 as u32;
                        }
                    }
                    if let Some(max_value1) = field_attr.get("decimal_places") {
                        if let Some(max_value1) = max_value1.as_u64() {
                            decimal_places = max_value1 as u32;
                        }
                    }

                    if decimal_places > 0 || (min_decimal_places == 0 && max_decimal_places == 0) {
                        // 如果decimal_places设置了 或者 所有值都没有设置，那么默认就是两位小数
                        if decimal_places <= 0 {
                            decimal_places = 2;
                        }
                        let x = float!(min_value, max_value, decimal_places);
                        result.insert(field_key.clone(), Value::from(x));
                    } else {
                        if min_decimal_places == 0 {
                            min_decimal_places = 2;
                        }
                        if max_decimal_places == 0 {
                            max_decimal_places = 16;
                        }

                        let x = float!(min_value, max_value, min_decimal_places, max_decimal_places);
                        result.insert(field_key.clone(), Value::from(x));
                    }
                }
                "timestamp" => {
                    let mut min_value = 0;
                    let mut max_value = 0; // 2299年，12月 31日 12时 12 分 12秒

                    if let Some(min_value1) = field_attr.get("min_value") {
                        if let Some(min_value1) = min_value1.as_u64() {
                            min_value = min_value1;
                        }
                    }

                    if let Some(max_value1) = field_attr.get("max_value") {
                        if let Some(max_value1) = max_value1.as_u64() {
                            max_value = max_value1;
                        }
                    }

                    let x = timestamp!(min_value, max_value);
                    result.insert(field_key.clone(), Value::from(x));
                }
                "number" | "int" | "posint" | "negint" => {
                    let mut min_value = i64::min_value();
                    let mut max_value = i64::max_value();
                    match field_type {
                        "posint" => {
                            min_value = 0;
                        }
                        "negint" => {
                            max_value = 0;
                        }
                        _ => ()
                    }

                    if let Some(min_value1) = field_attr.get("min_value") {
                        if let Some(min_value1) = min_value1.as_i64() {
                            min_value = min_value1;
                        }
                    }

                    if let Some(max_value1) = field_attr.get("max_value") {
                        if let Some(max_value1) = max_value1.as_i64() {
                            max_value = max_value1;
                        }
                    }
                    let x = int!(min_value, max_value);
                    result.insert(field_key.clone(), Value::from(x));
                }
                "date" | "datetime" => {
                    let mut min_value = "";
                    let mut max_value = "";

                    if let Some(min_value1) = field_attr.get("min_value") {
                        if let Some(min_value1) = min_value1.as_str() {
                            min_value = min_value1;
                        }
                    }

                    if let Some(max_value1) = field_attr.get("max_value") {
                        if let Some(max_value1) = max_value1.as_str() {
                            max_value = max_value1;
                        }
                    }
                    let d = if field_type == "date" {
                        mock::basic::datetime(min_value, max_value, "%Y-%m-%d")
                    } else {
                        mock::basic::datetime(min_value, max_value, "")
                    };

                    result.insert(field_key.clone(), Value::from(d));
                }
                "name" => {
                    result.insert(field_key.clone(), Value::String(mock::name::name()));
                }
                "cname" => {
                    result.insert(field_key.clone(), Value::String(mock::name::cname()));
                }
                "domain" => {
                    result.insert(field_key.clone(), Value::String(mock::web::domain(true)));
                }
                "ip" => {
                    result.insert(field_key.clone(), Value::String(mock::web::ip()));
                }
                "email" => {
                    result.insert(field_key.clone(), Value::String(mock::web::email()));
                }
                "url" => {
                    result.insert(field_key.clone(), Value::String(mock::web::url()));
                }
                "uuid" => {
                    result.insert(field_key.clone(), Value::String(mock::basic::uuid()));
                }
                "bool" => {
                    result.insert(field_key.clone(), Value::Bool(mock::basic::bool()));
                }
                "regex" => {
                    let mut r = "";
                    if let Some(v) = field_attr.get("regex") {
                        if let Some(v) = v.as_str() {
                            r = v.trim();
                        }
                    }
                    if r == "" {
                        result.insert(field_key.clone(), Value::String("".to_string()));
                    } else {
                        result.insert(field_key.clone(), Value::String(mock::basic::regex_string(r)));
                    }
                }
                "image" => {
                    let mut size = "";
                    let mut foreground = "";
                    let mut background = "";
                    let mut format = "";
                    let mut text = "";
                    if let Some(v) = field_attr.get("size") {
                        if let Some(v) = v.as_str() {
                            size = v;
                        }
                    }

                    if let Some(v) = field_attr.get("foreground") {
                        if let Some(v) = v.as_str() {
                            foreground = v;
                        }
                    }

                    if let Some(v) = field_attr.get("background") {
                        if let Some(v) = v.as_str() {
                            background = v;
                        }
                    }

                    if let Some(v) = field_attr.get("format") {
                        if let Some(v) = v.as_str() {
                            format = v;
                        }
                    }

                    if let Some(v) = field_attr.get("text") {
                        if let Some(v) = v.as_str() {
                            text = v;
                        }
                    }

                    result.insert(field_key.clone(), Value::String(mock::basic::image(size, foreground, background, format, text)));
                }
                "object" => {
                    let v = create_mock_value(field_attr);
                    result.insert(field_key.clone(), Value::Object(v));
                }
                "array" => {
                    if let Some(field_attr_array) = field_attr.as_array() {
                        if field_attr_array.len() > 0 {
                            let field_attr_one = &field_attr_array[0];
                            let field_type2 = get_field_type(field_attr_one);

                            let mut length = 0;
                            let mut min_length = 3;
                            let mut max_length = 10;
                            // 可以设定数组要展示多少个元素
                            if let Some(v) = field_attr_one.get("$length") {
                                if let Some(v) = v.as_u64() {
                                    length = v;
                                }
                            }

                            if let Some(v) = field_attr_one.get("$min_length") {
                                if let Some(v) = v.as_u64() {
                                    min_length = v;
                                }
                            }

                            if let Some(v) = field_attr_one.get("$max_length") {
                                if let Some(v) = v.as_u64() {
                                    max_length = v;
                                }
                            }
                            if length == 0 {
                                length = rng.gen_range(min_length, max_length);
                            }

                            match field_type2.to_lowercase().as_str() {
                                "object" => {
                                    let mut vec = Vec::with_capacity(length as usize);
                                    while length > 0 {
                                        let v = create_mock_value(field_attr_one);
                                        vec.push(Value::Object(v));
                                        length -= 1;
                                    }
                                    result.insert(field_key.clone(), Value::Array(vec));
                                }
                                "array" | _ => {
                                    let mut result2: Map<String, Value> = Map::new();
                                    result2.insert("key".to_string(), field_attr_one.clone());
                                    let mut vec = Vec::with_capacity(length as usize);
                                    while length > 0 {
                                        let v = create_mock_value(&Value::Object({ &result2 }.clone()));
                                        if v.contains_key("key") {
                                            vec.push(v["key"].clone());
                                        }
                                        length -= 1;
                                    }

                                    result.insert(field_key.clone(), Value::Array(vec));
                                }
                            }
                        }
                    }
                }
                "string" | "cword" | "cw" | "ctitle" | "ct" | "csentence" | "cs" | "csummary" | "cm" | "cparagraph" | "cp" | "word" | "title" | "sentence" | "summary" | "paragraph" | _ => {
                    get_string_value!(field_key, field_type, field_attr, result);
                }
            }
        }
    }

    result
}



