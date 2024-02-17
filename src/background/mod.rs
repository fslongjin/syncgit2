use std::{
    collections::HashSet,
    sync::{atomic::AtomicBool, Mutex},
    thread::JoinHandle,
    time::Duration,
};

use log::info;

use crate::{config_parser::TaskId, constant::REPO_CLONE_DIR};

use self::types::Event;

mod gitsync;
pub mod types;

lazy_static! {
    static ref BACKGROUND_QUEUE: thingbuf::ThingBuf<Event> = thingbuf::ThingBuf::new(100);
    static ref RUNNING_TASKS: Mutex<HashSet<TaskId>> = Mutex::new(HashSet::new());
}

pub static BG_SHOULD_STOP: AtomicBool = AtomicBool::new(false);

pub fn push_event(event: Event) {
    let r = BACKGROUND_QUEUE.push(event);
    if r.is_err() {
        log::error!("Failed to push event: {:?}", r.err());
    }
}

fn ensure_dir_exists(path: &str) {
    if !std::path::Path::new(path).exists() {
        std::fs::create_dir_all(path).unwrap();
    }
}

pub fn run_background() {
    ensure_dir_exists(REPO_CLONE_DIR);
    let mut handles: Vec<JoinHandle<()>> = vec![];
    loop {
        std::thread::sleep(Duration::from_millis(500));

        let event = BACKGROUND_QUEUE.pop();
        if event.is_none() {
            if BG_SHOULD_STOP.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            continue;
        }

        info!("event: {:?}", event);

        if BG_SHOULD_STOP.load(std::sync::atomic::Ordering::SeqCst) {
            info!(
                "Stopping background, remaining tasks: {}",
                BACKGROUND_QUEUE.len()
            );
        }
        let event = event.unwrap();

        let mut running_tasks = RUNNING_TASKS.lock().unwrap();
        if event.is_valid() {
            if !running_tasks.contains(event.id()) {
                log::info!("Starting worker for task: {:?}", event.id());
                running_tasks.insert(event.id().clone());

                drop(running_tasks);
                let worker = Worker::new(event.id().clone(), event);
                let handle = worker.run();
                handles.push(handle);
            } else {
                log::info!("Task already running: {:?}", event.id());
                BACKGROUND_QUEUE.push(event).ok();
                continue;
            }
        }
    }
}

struct Worker {
    task_id: TaskId,
    event: Event,
}

impl Worker {
    fn new(task_id: TaskId, event: Event) -> Worker {
        Worker { task_id, event }
    }

    fn run(self) -> JoinHandle<()> {
        let r = std::thread::spawn(move || {
            log::info!("Running worker for task: {:?}", self.task_id);
            // do some work

            gitsync::sync_git(self.task_id.clone())
                .map_err(|e| {
                    log::error!("Failed to sync {:?}: {:?}", self.task_id, e);
                })
                .ok();

            let mut running_tasks = RUNNING_TASKS.lock().unwrap();
            running_tasks.remove(&self.task_id);
            drop(running_tasks);
        });

        return r;
    }
}
