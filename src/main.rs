#![feature(proc_macro_hygiene, decl_macro)]

extern crate anyhow;
extern crate dotenv_codegen;
extern crate rocket;
extern crate serde;

use log4rs;
use log::warn;

use tesla_metrics::poller::Poller;

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap_or_else(|e| {
        warn!("Failed to load log4rs.yaml, {}", e);
    });

    rocket::ignite().attach(Poller::fairing()).launch();
}
