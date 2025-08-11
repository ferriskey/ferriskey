use std::sync::{Arc, LazyLock, Mutex};

use test_context::AsyncTestContext;

use crate::docker::*;

fn start_db() -> String {
    let args = [
        "run",
        "-d",
        "-e",
        "POSTGRES_PASSWORD=password",
        "-p",
        "5432:5432",
        "postgres:latest",
    ];
    start_container("docker", &args, |id| {
        std::process::Command::new("docker")
            .args(["exec", id, "pg_isready", "-t", "90"])
            .output()
            .unwrap()
            .status
            .success()
    })
}

struct DB {
    id: String,
}

impl DB {
    fn new() -> Self {
        Self { id: start_db() }
    }
}

impl Drop for DB {
    fn drop(&mut self) {
        stop_container(&self.id)
    }
}

static DB_INSTANCE: LazyLock<Mutex<Option<Arc<DB>>>> = LazyLock::new(|| Mutex::new(None));

pub struct DBContext {
    _id: Arc<DB>,
}

impl DBContext {
    pub fn url(&self) -> &'static str {
        "postgresql://postgres:password@localhost"
    }
}

impl AsyncTestContext for DBContext {
    async fn setup() -> DBContext {
        let mut is_new = false;
        let _id = {
            let mut lock = DB_INSTANCE.lock().unwrap();
            if let Some(db) = &*lock {
                db.clone()
            } else {
                is_new = true;
                let db = Arc::new(DB::new());
                *lock = Some(db.clone());
                db
            }
        };

        if is_new {
            // todo: migrate it
            //let _ = migrate("postgresql://postgres:password@localhost")
            //    .await
            //    .unwrap();
        }

        DBContext { _id }
    }

    async fn teardown(self) {
        let mut lock = DB_INSTANCE.lock().unwrap();
        let arc = lock.as_ref();
        if let Some(arc) = arc {
            if Arc::strong_count(arc) == 2 {
                *lock = None; // Remove from static; since static does not call Drop
            }
        }
    }
}
