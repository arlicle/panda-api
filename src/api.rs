use actix_web::{http, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value, Map};
use std::collections::HashMap;

use std::sync::Mutex;
use log::debug;
use crate::db::{self, ApiDoc};


#[derive(Serialize, Deserialize, Debug)]
struct DocSummary {
    pub name: String,
    pub desc: String,
    pub order: i64,
    pub filename: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ApiDocDataRequest {
    filename: String,
}


pub fn server_info() -> HttpResponse {
    HttpResponse::Ok().json(json!({
      "name": "mockrs",
      "author": "PrivateRookie"
    }))
}


/// 根据接口文件路径获取接口文档详情
pub fn get_api_doc_data(req: HttpRequest, req_get: web::Query<ApiDocDataRequest>, data: web::Data<Mutex<db::Database>>) -> HttpResponse {
    let mut data = data.lock().unwrap();
    let mut api_docs = &data.api_docs;

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
pub fn get_api_doc_basic(req: HttpRequest, data: web::Data<Mutex<db::Database>>) -> HttpResponse {
    let mut data = data.lock().unwrap();
    let mut basic_data = &data.basic_data;
    let mut api_docs = &data.api_docs;

    let mut docs = Vec::new();
    for (_, doc) in api_docs {
        docs.push(DocSummary { name: doc.name.clone(), desc: doc.desc.clone(), order: doc.order, filename: doc.filename.clone() });
    }

    HttpResponse::Ok().json(json!({
      "project_name": &basic_data.project_name,
      "project_desc": &basic_data.project_desc,
      "read_me": &basic_data.read_me,
      "api_docs": docs
    }))
}


/// 处理get请求
pub fn do_get(req: HttpRequest, req_get: Option<web::Query<Value>>, db_data: web::Data<Mutex<db::Database>>) -> HttpResponse {
    let req_get = match req_get {
        Some(x) => x,
        None => web::Query(Value::Null)
    };
    let req_get = req_get.as_object().unwrap();


    find_response_data(&req, req_get, db_data)
}


/// 处理post、put、delete 请求
///
pub fn do_post(req: HttpRequest, request_data: Option<web::Json<Value>>, db_data: web::Data<Mutex<db::Database>>) -> HttpResponse {
    let request_data = match request_data {
        Some(x) => x,
        None => web::Json(Value::Null)
    };
    let request_data = request_data.as_object().unwrap();

    find_response_data(&req, request_data, db_data)
}


/// 找到对应url 对应请求的数据
///
fn find_response_data(req: &HttpRequest, request_data: &Map<String, Value>, db_data: web::Data<Mutex<db::Database>>) -> HttpResponse {
    let db_data = db_data.lock().unwrap();
//    let data = data.lock().expect("jjjjjjjjjjjjjjjjjjjj");
    let api_data = &db_data.api_data;
    let req_path = req.path();

    let req_method = req.method().as_str();
    match api_data.get(req_path) {
        Some(x) => {

            let api_data = x.get(req_method).unwrap();
//            let api_data = api_data.lock().unwrap();
//
//            let test_data = &api_data.test_data;
//            let test_data = test_data.as_array().unwrap();


        }
        None => println!("404")
    };

    HttpResponse::Ok().json(json!({
      "code": -1,
      "msg": format!("this api address not defined {}", req_path)
    }))
}