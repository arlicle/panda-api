use actix::Addr;
use actix_files;
use actix_multipart::Multipart;
use actix_web::dev::ResourceDef;
use actix_web::{http, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};
use std::thread;

use futures::StreamExt;
use rand::{thread_rng, Rng};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::db;
use crate::mock;
use crate::server;
use crate::websocket::WsChatSession;
use crate::{float, int, timestamp};

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiDocDataRequest {
    filename: String,
}

/// 根据接口文件路径获取接口文档详情
pub async fn get_api_doc_data(
    req_get: web::Query<ApiDocDataRequest>,
    data: web::Data<Mutex<db::Database>>,
) -> HttpResponse {
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

    if req_get.filename.ends_with(".md") {
        if Path::new(&req_get.filename).exists() {
            let mut order = 0;
            let menu_title = "".to_string();
            let desc = "".to_string();
            let md_content = "".to_string();
            let filename = "".to_string();
            let (order, menu_title, _, md_content, _) = db::load_md_doc_config(
                &req_get.filename,
                order,
                menu_title,
                desc,
                md_content,
                filename,
            );
            return HttpResponse::Ok().json(json!({
                    "order": order,
                    "name": menu_title,
                    "content": md_content}));
        }
    } else if req_get.filename.ends_with(".json5") {
        for (_, doc) in api_docs {
            if doc.filename == req_get.filename {
                let mut apis = Vec::new();
                for api in &doc.apis {
                    let api = api.lock().unwrap();
                    apis.push(api.clone());
                }
                return HttpResponse::Ok().json(json!({
                    "name": doc.name,
                    "desc": doc.desc,
                    "order": doc.order,
                    "filename": doc.filename,
                    "apis": apis}));
            }
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
pub async fn get_api_doc_basic(db_data: web::Data<Mutex<db::Database>>) -> HttpResponse {
    let data = db_data.lock().unwrap();
    let basic_data = &data.basic_data;

    let mut docs = Vec::new();
    if let Some(auth_doc) = &data.auth_doc {
        docs.push(json!({"name":auth_doc.name, "filetype":"auth", "desc":auth_doc.desc, "order":0, "filename":"_auth.json5", "children":{}}));
    }
    if let Some(_) = &data.settings {
        docs.push(json!({"name":"Settings", "filetype":"settings", "desc":"", "order":0, "filename":"_settings.json5", "children":{}}));
    }

    for (_, doc) in &data.menus {
        docs.push(json!({ "name": doc.name, "filetype":doc.filetype, "desc": doc.desc, "order": doc.order, "filename": doc.filename, "children":doc.children }));
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
    let theme_home_dir = format!(
        "{}/.panda_api/theme",
        home_dir.to_str().unwrap().trim_end_matches("/")
    );
    let theme_file;
    if req_path == "/" {
        theme_file = "/index.html";
    } else {
        theme_file = req_path.trim_start_matches("/__api_docs/theme");
    }

    // 优先加载本地目录皮肤，如果本地目录皮肤不存在，加载安装目录皮肤
    let theme_filepath = format!("_theme{}", theme_file);
    if Path::new(&theme_filepath).exists() {
        return Ok(actix_files::NamedFile::open(theme_filepath)?);
    }

    // 加载安装目录的皮肤
    let theme_filepath = format!("{}{}", theme_home_dir, theme_file);
    Ok(actix_files::NamedFile::open(theme_filepath)?)
}

/// 获取用户自己存放的静态文件
/// 多用于写markdown的时候存放的图片
pub async fn static_file_view(req: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    let req_path = req.path().trim_start_matches("/");
    return Ok(actix_files::NamedFile::open(req_path)?);
}

/// 查看上传的 图片或文件
pub async fn upload_file_view(req: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    let req_path = req.path();
    let file_path = "_data".to_string() + req_path;
    return Ok(actix_files::NamedFile::open(file_path)?);
}

/// 获取_data目录中的数据, models数据 或者其它加载数据
pub async fn get_api_doc_schema_data(req_get: web::Query<ApiDocDataRequest>) -> HttpResponse {
    let read_me = match fs::read_to_string(&req_get.filename) {
        Ok(x) => x,
        Err(_) => "no data file".to_string(),
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .body(read_me)
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
pub async fn action_handle(
    req: HttpRequest,
    request_body: Option<web::Json<Value>>,
    request_query: Option<web::Query<Value>>,
    request_form_data: Option<Multipart>,
    db_data: web::Data<Mutex<db::Database>>,
) -> HttpResponse {
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
        Some(x) => x.into_inner(),
        None => Value::Null,
    };

    let request_query = match request_query {
        Some(x) => x.into_inner(),
        None => Value::Null,
    };

    find_response_data(
        &req,
        body_mode,
        request_body,
        request_query,
        form_data,
        db_data,
    )
}

/// 找到对应url 对应请求的数据
///
fn find_response_data(
    req: &HttpRequest,
    body_mode: String,
    request_body: Value,
    request_query: Value,
    form_data: Value,
    db_data: web::Data<Mutex<db::Database>>,
) -> HttpResponse {
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
            if a_api_data.method.contains(&req_method.to_string())
                || a_api_data.method.contains(&"*".to_string())
            {
                if a_api_data.auth {
                    // 权限检查
                    if let Some(auth_valid_errors) =
                    auth_validator(&req, &a_api_data.url, &db_data.auth_doc)
                    {
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
                let response_type = db::get_field_type(&a_api_data.response);

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
                            None => &Value::Null,
                        };
                        if !is_value_equal(&request_body, v) {
                            continue;
                        }

                        let v = match test_case_data.get("form-data") {
                            Some(v) => v,
                            None => &Value::Null,
                        };
                        if !is_value_equal(&form_data, v) {
                            continue;
                        }

                        let v = match test_case_data.get("query") {
                            Some(v) => v,
                            None => &Value::Null,
                        };
                        let request_query = parse_request_query_to_api_query_format(
                            &request_query,
                            &a_api_data.query,
                        );
                        if !is_value_equal(&request_query, v) {
                            continue;
                        }

                        let case_response = match test_case_data.get("response") {
                            Some(v) => v,
                            None => &Value::Null,
                        };

                        let response = parse_test_case_response(case_response, "", &a_api_data.response);
                        if let Some(v) = test_case_data.get("delay") {
                            if let Some(t) = v.as_u64() {
                                thread::sleep(Duration::from_millis(t));
                            }
                        }

                        let serialized = serde_json::to_string(&response).unwrap();
                        return HttpResponse::build(status_code)
                            .content_type(content_type)
                            .body(serialized);
                    }
                }

                let mut serialized = "".to_string();
                if let Some(response) = create_mock_value(&a_api_data.response, "", &a_api_data.response) {
                    serialized = serde_json::to_string(&response).unwrap();
                }
                return HttpResponse::build(status_code)
                    .content_type(content_type)
                    .body(serialized);
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
fn parse_test_case_response(
    test_case_response: &Value,
    field_path: &str,
    response_model: &Value,
) -> Value {
    if test_case_response.is_null() {
        return Value::Null;
    }
    let mut result = Map::new();
    match test_case_response {
        Value::Object(test_response) => {
            for (field_key, field) in test_response {
                match field {
                    Value::Object(field_obj) => {
                        // 拿到$mock设定的字段，并且值要为true
                        if let Some(v) = field_obj.get("$mock") {
                            if let Some(true) = v.as_bool() {
                                // 首先拿出对应response字段的设置
                                let pointer = format!("{}/{}", field_path, field_key);
                                if let Some(model_field) = response_model.pointer(&pointer) {
                                    let mut new_model_field_attr: Map<String, Value> =
                                        Map::new();
                                    if let Some(model_field_obj) = model_field.as_object() {
                                        // 先获取对应response字段的属性
                                        new_model_field_attr = model_field_obj.clone();
                                        // 用当前新属性进行值的重写
                                        for (k2, v2) in field_obj {
                                            if k2 == "$mock" {
                                                continue;
                                            }
                                            new_model_field_attr
                                                .insert(k2.to_string(), v2.clone());
                                        }
                                    }

                                    let v_obj = Value::Object(new_model_field_attr);
                                    if let Some(v) = create_mock_value(&v_obj, "", &v_obj){
                                        result.insert(field_key.to_string(), v);
                                    }
                                }
                            }
                        } else {
                            let pointer = format!("{}/{}", field_path, field_key);
                            let v = parse_test_case_response(field, &pointer, response_model);
                            result.insert(field_key.to_string(), v);
                        }
                    }
                    Value::Array(field_array) => {
                        if field_array.len() >= 1 {
                            let v = &field_array[0];
                            let pointer = format!("{}/{}/0", field_path, field_key);
                            let v = parse_test_case_response(v, &pointer, response_model);
                            result.insert(field_key.to_string(), v);
                        }
                    }
                    _ => {
                        result.insert(field_key.to_string(), field.clone());
                    }
                }
            }
        }
        Value::Array(field_array) => {
            let mut array_result = Vec::new();
            for item in field_array {
                let pointer = format!("{}/0", field_path);
                let v = parse_test_case_response(item, &pointer, response_model);
                array_result.push(v);
            }
            return Value::Array(array_result);
        }
        _ => {
            return test_case_response.clone();
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
                                    result.insert(
                                        field_key.to_string(),
                                        json!(request_query_value_str),
                                    );
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
        Value::Array(value1_array) => match value2.as_array() {
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
        },
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
                            form_data.insert(
                                field_name.to_string(),
                                Value::String(filename.to_string()),
                            );
                            form_data.insert(
                                format!("__{}", field_name),
                                Value::String(format!("/_upload/{}", filename)),
                            );
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
        None => return false,
    }

    false
}

/// 判断用户是否有当前接口访问权限，如果有权限返回None，如果没有权限 返回报错信息
fn auth_validator<'a>(
    req: &HttpRequest,
    api_url: &str,
    auth_doc: &'a Option<db::AuthDoc>,
) -> Option<&'a Value> {
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


/// 获取string类型的mock value
fn get_string_mock_value(field_type: &str, field_attr: &Value) -> Value {
    let mut min_length = 0;
    let mut max_length = 0;
    let mut length = 0;
    let mut content_type = "markdown";

    if let Some(min_value1) = field_attr.get("length") {
        if let Some(min_value1) = min_value1.as_u64() {
            length = min_value1;
        }
    }

    if let Some(min_value1) = field_attr.get("min_length") {
        if let Some(min_value1) = min_value1.as_u64() {
            min_length = min_value1;
        }
    }

    if let Some(max_value1) = field_attr.get("max_length") {
        if let Some(max_value1) = max_value1.as_u64() {
            max_length = max_value1;
        }
    }

    if let Some(min_value1) = field_attr.get("content_type") {
        if let Some(min_value1) = min_value1.as_str() {
            content_type = min_value1;
        }
    }

    if max_length <= min_length {
        max_length = min_length + 3;
    }

    match field_type {
        "name" => {
            return Value::String(mock::name::name());
        }
        "cname" => {
            return Value::String(mock::name::cname());
        }
        "domain" => {
            return Value::String(mock::web::domain(true));
        }
        "ip" => {
            return Value::String(mock::web::ip());
        }
        "email" => {
            return Value::String(mock::web::email());
        }
        "url" => {
            return Value::String(mock::web::url());
        }
        "uuid" => {
            return Value::String(mock::basic::uuid());
        }
        "cword" | "cw" => {
            return Value::String(mock::text::cword(length as usize, min_length, max_length));
        }
        "ctitle" | "ct" => {
            return Value::String(mock::text::ctitle(length, min_length, max_length));
        }
        "csentence" | "cstring" | "cs" => {
            return Value::String(mock::text::csentence(length, min_length, max_length));
        }
        "csummary" | "cm" => {
            return Value::String(mock::text::csummary(length, min_length, max_length));
        }
        "cparagraph" | "cp" => {
            return Value::String(mock::text::cparagraph(
                length,
                min_length,
                max_length,
                content_type,
            ));
        }
        "word" => {
            return Value::String(mock::text::word(length as usize, min_length, max_length));
        }
        "title" => {
            return Value::String(mock::text::title(length, min_length, max_length));
        }
        "sentence" => {
            return Value::String(mock::text::sentence(length, min_length, max_length));
        }
        "summary" => {
            return Value::String(mock::text::summary(length, min_length, max_length));
        }
        "paragraph" => {
            return Value::String(mock::text::paragraph(
                length,
                min_length,
                max_length,
                content_type,
            ));
        }
        "string" | _ => {
            return Value::String(mock::basic::string(length, min_length, max_length));
        }
    }
}

/// 根据response定义生成返回给前端的mock数据
///
pub fn create_mock_value(response_model: &Value, rec_path: &str, org_response_model: &Value) -> Option<Value> {
    let mut rng = thread_rng();
    let response_type = db::get_field_type(response_model);
    let response_model_type = response_type.as_str();

    if is_marked_delete_field(response_model) {
        return None;
    }

    if !["object", "array", "map", "rec"].contains(&response_model_type) {
        // 只要不是数组 、对象、map、rec 这种结构节点，直接输出mock值
        return create_mock_value_by_field("", &rec_path, response_model, org_response_model);
    }

    if "rec" == response_model_type {
        if let Some(path) = response_model.get("$ref") {
            if let Some(path_str) = path.as_str() {
                if let Some(v) = create_recursive_mock_model(path_str, rec_path, response_model, org_response_model) {
                    return create_mock_value(&v, "", org_response_model);
                }
            }
        }
    }

    if "map" == response_model_type {
        let key_v = if let Some(v) = response_model.get("$key") {
            v
        } else {
            return None;
        };

        let value_v = if let Some(v) = response_model.get("$value") {
            v
        } else {
            return None;
        };

        let mut length = 0;
        let mut min_length = 0;
        let mut max_length = 7;
        if let Some(v) = response_model.get("$length") {
            if let Some(v) = v.as_u64() {
                length = v;
            }
        }
        if let Some(v) = response_model.get("$min_length") {
            if let Some(v) = v.as_u64() {
                min_length = v;
            }
        }
        if let Some(v) = response_model.get("$max_length") {
            if let Some(v) = v.as_u64() {
                max_length = v;
            }
        }
        if max_length <= min_length {
            max_length = min_length + 3;
        }
        if length == 0 {
            // 默认有5到10个句子
            length = rng.gen_range(min_length, max_length);
        }

        let rec_path1 = format!("{}/$key", rec_path);
        let rec_path2 = format!("{}/$value", rec_path);
        let mut result = Map::new();
        while length > 0 {
            length -= 1;
            let key = create_mock_value(key_v, &rec_path1, org_response_model);
            let value = create_mock_value(value_v, &rec_path2, org_response_model);
            if let Some(value) = value {
                if let Some(key) = key {
                    match key {
                        Value::String(k) => {
                            result.insert(k, value);
                        }
                        Value::Bool(k) => {
                            result.insert(k.to_string(), value);
                        }
                        Value::Number(k) => {
                            result.insert(k.to_string(), value);
                        }
                        _ => {}
                    }
                }
            }
        }

        return Some(Value::Object(result));
    }

    if let Some(response_model_obj) = response_model.as_object() {
        let mut result: Map<String, Value> = Map::new();
        for (field_key, field_attr) in response_model_obj {
            if is_special_private_key(field_key) {
                continue;
            }

            if is_marked_delete_field(field_attr) {
                continue;
            }

            let rec_path = format!("{}/{}", rec_path, field_key);
            if let Some(value) = create_mock_value(field_attr, &rec_path, org_response_model) {
                result.insert(field_key.to_string(), value);
            }
        }
        return Some(Value::Object(result));
    }

    if let Some(response_model_array) = response_model.as_array() {
        let mut array_vec = Vec::new();
        if response_model_array.len() > 0 {
            for (index, field_attr_one) in response_model_array.iter().enumerate() {
                if field_attr_one.is_null() {
                    continue;
                }
                let index = index.to_string();
                let field_type2 = db::get_field_type(field_attr_one);
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
                if max_length <= min_length {
                    max_length = min_length + 3;
                }
                if length == 0 {
                    length = rng.gen_range(min_length, max_length + 1);
                }

                let mut new_rec_path = format!("{}/{}", rec_path, index);
                while length > 0 {
                    if let Some(v) = create_mock_value(field_attr_one, &new_rec_path, org_response_model){
                        array_vec.push(v);
                    }
                    length -= 1;
                }
            }
        }
        return Some(Value::Array(array_vec));
    }

    None
}


/// 判断这个字段是否是系统的特殊私有字段
fn is_special_private_key(field_key: &str) -> bool {
    if field_key == "$type"
        || field_key == "$name"
        || field_key == "$desc"
        || field_key == "$ref"
        || field_key == "$length"
        || field_key == "$min_length"
        || field_key == "$max_length"
        || field_key == "$required"
    {
        return true;
    }
    return false;
}

/// 根据字段和字段属性，生成mock数据
fn create_mock_value_by_field(field_key: &str, rec_path: &str, field_attr: &Value, org_response_model: &Value) -> Option<Value> {
    if is_special_private_key(field_key)
        || field_attr.is_null()
    {
        return None;
    }

    let mut rng = thread_rng();
    let field_type = db::get_field_type(field_attr);
    let field_type = field_type.as_str();

    let mut required = true;

    match field_attr.get("required") {
        Some(v) => {
            if let Some(v) = v.as_bool() {
                required = v;
            }
        }
        None => (),
    }

    if !required {
        // 如果required是false，那么返回数据就随机丢失
        let n = rng.gen_range(0, 10);
        if n % 2 == 0 {
            return None;
        }
    }

    if let Some(value1) = field_attr.get("value") {
        // 如果设定了value，那么就只返回一个固定的值
        return Some(value1.clone());
    }

    if let Some(enum_data) = field_attr.get("enum") {
        // 如果设置了枚举值，那么就只使用枚举值
        let list = enum_data.as_array().unwrap();
        if list.len() == 0 {
            return Some(Value::Null);
        }
        let n = rng.gen_range(0, list.len());
        let v = &list[n];
        if let Some(v2) = v.pointer("/$value") {
            return Some(v2.clone());
        }
        return Some(Value::Null);
    }

    if ["float", "posfloat", "negfloat"].contains(&field_type) {
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
            _ => (),
        }

        if let Some(v) = field_attr.get("min_value") {
            if let Some(v) = v.as_f64() {
                min_value = v;
            }
        }

        if let Some(v) = field_attr.get("max_value") {
            if let Some(v) = v.as_f64() {
                max_value = v;
            }
        }

        if let Some(v) = field_attr.get("min_decimal_places") {
            if let Some(v) = v.as_u64() {
                min_decimal_places = v as u32;
            }
        }
        if let Some(v) = field_attr.get("max_decimal_places") {
            if let Some(v) = v.as_u64() {
                max_decimal_places = v as u32;
            }
        }
        if let Some(v) = field_attr.get("decimal_places") {
            if let Some(v) = v.as_u64() {
                decimal_places = v as u32;
            }
        }

        if decimal_places > 0 || (min_decimal_places == 0 && max_decimal_places == 0) {
            // 如果decimal_places设置了 或者 所有值都没有设置，那么默认就是两位小数
            if decimal_places <= 0 {
                decimal_places = 2;
            }
            let x = float!(min_value, max_value, decimal_places);
            return Some(Value::from(x));
        } else {
            if min_decimal_places == 0 {
                min_decimal_places = 2;
            }
            if max_decimal_places == 0 {
                max_decimal_places = 16;
            }

            let x =
                float!(min_value, max_value, min_decimal_places, max_decimal_places);
            return Some(Value::from(x));
        }
    }

    if "timestamp" == field_type {
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
        return Some(Value::from(x));
    }

    if ["number", "int", "posint", "negint"].contains(&field_type) {
        let mut min_value = i64::min_value();
        let mut max_value = i64::max_value();
        match field_type {
            "posint" => {
                min_value = 0;
            }
            "negint" => {
                max_value = 0;
            }
            _ => (),
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
        return Some(Value::from(x));
    }

    if ["date", "datetime"].contains(&field_type) {
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

        return Some(Value::from(d));
    }

    if ["bool"].contains(&field_type) {
        return Some(Value::Bool(mock::basic::bool()));
    }
    if ["regex"].contains(&field_type) {
        let mut r = "";
        if let Some(v) = field_attr.get("regex") {
            if let Some(v) = v.as_str() {
                r = v.trim();
            }
        }
        if r == "" {
            return Some(Value::String("".to_string()));
        } else {
            return Some(Value::String(mock::basic::string_from_regex(r)));
        }
    }

    if ["image"].contains(&field_type) {
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

        return Some(Value::String(mock::basic::image(
            size, foreground, background, format, text,
        )));
    }

    // 其它字段一律按字符串处理
    // "string" | "cword" | "cw" | "ctitle" | "ct" | "csentence" | "cstring" | "cs"
    //        | "csummary" | "cm" | "cparagraph" | "cp" | "word" | "title" | "sentence"
    //        | "summary" | "paragraph"

    return Some(get_string_mock_value(field_type, field_attr));
}

/// 生成递归结构的mock数据
/// rec_model_path_str 要递归的value
/// rec_pointer_path_str 递归重复指向的节点
fn create_recursive_mock_model(rec_model_path_str: &str, rec_pointer_path_str: &str, field_attr: &Value, org_response_model: &Value) -> Option<Value> {
    if !rec_model_path_str.starts_with("/") {
        // 递归结构的数据路径只允许递归response内部的，必须以$response开头
        return None;
    }
    let mut rng = thread_rng();

    let mut empty_value_conf: Option<Value> = None;
    let mut length = 0;
    let mut min_length = 0;
    let mut max_length = 4;
    let mut count = 0;
    let mut min_count = 0;
    let mut max_count = 4;

    // 可以设定数组要展示多少个元素
    if let Some(v) = field_attr.get("$empty_value") {
        empty_value_conf = Some(v.clone());
    }

    // 可以设定数组要展示多少个元素
    if let Some(v) = field_attr.get("$length") {
        if let Some(v) = v.as_u64() {
            length = v;
        }
    }

    if let Some(v) = field_attr.get("$min_length") {
        if let Some(v) = v.as_u64() {
            min_length = v;
        }
    }

    if let Some(v) = field_attr.get("$max_length") {
        if let Some(v) = v.as_u64() {
            max_length = v;
        }
    }

    if max_length <= min_length {
        max_length = min_length + 3;
    }
    if length == 0 {
        length = rng.gen_range(min_length, max_length);
    }

    // 可以设定递归要展示多少层
    if let Some(v) = field_attr.get("$count") {
        if let Some(v) = v.as_u64() {
            count = v;
        }
    }

    if let Some(v) = field_attr.get("$min_count") {
        if let Some(v) = v.as_u64() {
            min_count = v;
        }
    }

    if let Some(v) = field_attr.get("$max_count") {
        if let Some(v) = v.as_u64() {
            max_count = v;
        }
    }
    if max_count <= min_count {
        max_count = min_count + 3;
    }
    if count == 0 {
        count = rng.gen_range(min_count, max_count);
    }

    // 拿到递归模型
    let mut mock_model;
    let mut rec_pointer_path_string;
    if rec_model_path_str == "/" {
        // 如果直接是/ 代表直接使用根节点，
        mock_model = org_response_model.clone();
        rec_pointer_path_string = rec_pointer_path_str.to_string();
    } else {
        if let Some(rec_model) = org_response_model.pointer(rec_model_path_str) {
            mock_model = rec_model.clone();
            rec_pointer_path_string = rec_pointer_path_str.trim_start_matches(rec_model_path_str).to_string();
        } else {
            return None;
        }
    }

    let rec_model = mock_model.clone();

    let field_type = db::get_field_type(&rec_model);
    let field_type = field_type.as_str();

    let relative_path = rec_pointer_path_string.clone();
    // 生成mock模型
    while count > 0 {
        if let Some(model_v) = mock_model.pointer_mut(&rec_pointer_path_string) {
            *model_v = rec_model.clone();
            rec_pointer_path_string = rec_pointer_path_string + &relative_path;
        }
        count -= 1;
    }
    // 递归的尾节点设置为
    if let Some(model_v) = mock_model.pointer_mut(&rec_pointer_path_string) {
        // 如果用户设置了默认的空值，那么使用用户定义的
        if let Some(empty_v) = empty_value_conf {
            let mut m = Map::new();
            m.insert("type".to_string(), Value::String("string".to_string()));
            m.insert("value".to_string(), empty_v);
            *model_v = Value::Object(m);
        } else {
            // 如果用户未定义，那么按类型来
            *model_v = match field_type {
                "array" => json!([]),
                "object" => Value::Null,
                "map" => json!({"type":"string","value":{}}),
                _ => {
                    Value::Null
                }
            }
        }
    }
    return Some(mock_model);
}

/// 检查value字段中是否标记了$del:true
fn is_marked_delete_field(field_value: &Value) -> bool {
    if let Some(del) = field_value.pointer("/$del") {
        // 如果标记了删除，那么就返回空
        if let Some(true) = del.as_bool() {
            return true;
        }
    }
    return false;
}