#![feature(proc_macro_hygiene, decl_macro)]

extern crate anyhow;
extern crate rocket;
extern crate serde;

use dotenv::dotenv;
use log::{info, warn};
use log4rs;

use tesla_metrics::poller::Poller;

fn main() {
    dotenv().ok();

    if let Err(e) = log4rs::init_file("log4rs.yaml", Default::default()) {
        warn!("Failed to load log4rs.yaml, {}", e);
    }

    info!("Starting up!!!");

    rocket::ignite().attach(Poller::fairing()).launch();
}
