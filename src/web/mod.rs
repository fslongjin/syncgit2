use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Scope};
use log::info;

mod github;
use crate::web::github::github_webhook;

pub(super) async fn run_webapp() -> actix_web::dev::Server {
    log::info!("Starting webapp");

    const ADDR: &str = "0.0.0.0";
    const PORT: u16 = 10089;

    let r: actix_web::dev::Server = HttpServer::new(|| {
        App::new()
            .service(index)
            .service(Scope::new("/webhook").service(github_webhook))
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .workers(2)
    .bind((ADDR, PORT))
    .unwrap()
    .run();

    info!("Webapp started at http://{}:{}", ADDR, PORT);

    return r;
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("syncgit2 server")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
