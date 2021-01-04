use std::time::Duration;

use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use once_cell::sync::Lazy;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::Rocket;
use rocket_prometheus::{
    prometheus::{opts, IntCounterVec},
    PrometheusMetrics,
};

static NAME_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    IntCounterVec::new(opts!("name_counter", "Count of names"), &["name"])
        .expect("Could not create lazy IntCounterVec")
});

pub fn start_poller() -> ScheduleHandle {
    let mut scheduler = Scheduler::new();

    scheduler.every(5.second()).run(|| {
        NAME_COUNTER.with_label_values(&["foo-bar"]).inc();
    });

    scheduler.watch_thread(Duration::from_millis(100))
}

pub struct Poller;

impl Poller {
    pub fn fairing() -> Poller {
        Poller {}
    }
}

impl Fairing for Poller {
    fn info(&self) -> Info {
        Info {
            name: "Poller",
            kind: Kind::Attach,
        }
    }

    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        let prometheus = PrometheusMetrics::new();
        prometheus
            .registry()
            .register(Box::new(NAME_COUNTER.clone()))
            .unwrap();

        Ok(rocket
            .attach(prometheus.clone())
            .mount("/metrics", prometheus)
            .manage(start_poller()))
    }
}
