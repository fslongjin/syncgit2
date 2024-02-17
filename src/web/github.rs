use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use log::info;

use crate::{
    background::types::Event,
    config_parser::{all_tasks, TaskId},
};

#[post("/github/{task_id}")]
async fn github_webhook(
    path: web::Path<(String,)>,
    req_body: String,
    req: actix_web::HttpRequest,
) -> impl Responder {
    info!("github_webhook: {:?}", path.0);
    let config = all_tasks().get(&TaskId::new(&path.0));
    if config.is_none() {
        return HttpResponse::NotFound().body("Task not found");
    }

    let config = config.unwrap();

    let headers = req.headers();
    for (name, value) in headers.iter() {
        info!("{}: {:?}", name, value);
    }
    let target = headers.get("x-github-hook-installation-target-type");
    if target.is_none() {
        log::error!("x-github-hook-installation-target-type not found");
        return HttpResponse::BadRequest().body("x-github-hook-installation-target-type not found");
    }
    let target = target.unwrap().to_str().unwrap();
    if target != "repository" {
        log::error!("x-github-hook-installation-target-type is not repository");
        return HttpResponse::BadRequest()
            .body("x-github-hook-installation-target-type is not repository");
    }

    // 校验签名
    let signature = headers.get("x-hub-signature-256");

    let real_sha256: Option<String> = config.secret_sha256();
    if real_sha256.is_some() && signature.is_none() {
        log::error!("x-hub-signature-256 not found");
        return HttpResponse::BadRequest().body("x-hub-signature-256 not found");
    }

    let signature = signature.unwrap().to_str().unwrap();
    let signature = signature.strip_prefix("sha256=").unwrap();
    if let Some(sha256) = real_sha256 {
        info!("real_sha256: {}", sha256);
        if signature != sha256 {
            log::error!("x-hub-signature-256 not match");
            return HttpResponse::BadRequest().body("x-hub-signature-256 not match");
        }
    }

    // 添加任务
    let event = Event::new(config.task_id());
    crate::background::push_event(event);

    return HttpResponse::Ok().body("ok");
}
