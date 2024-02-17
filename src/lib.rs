use log::info;
use serde::ser;

use crate::background::{run_background, BG_SHOULD_STOP};

mod background;
mod cmdline;
mod config_parser;
mod constant;
mod web;

#[macro_use]
extern crate lazy_static;

/// 程序入口
pub fn run() {
    log4rs::init_file("config/log/log4rs.yml", Default::default()).unwrap();
    println!("Hello, world!");
    log::info!("Starting up");
    log::info!("Loading all tasks");
    let tasks = config_parser::all_tasks();

    let bg_handle = std::thread::spawn(move || {
        run_background();
    });

    actix::run(async move {
        let web = web::run_webapp();
        let server = web.await;
        server.await.ok();
        info!("Webapp stopped");
        info!("Stopping background");
        BG_SHOULD_STOP.store(true, std::sync::atomic::Ordering::SeqCst);

        actix::System::current().stop();
    })
    .expect("Failed to run actix");
    bg_handle.join().unwrap();
    info!("Background stopped");
}
