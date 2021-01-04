#![feature(proc_macro_hygiene, decl_macro)]

extern crate dotenv_codegen;
extern crate rocket;
extern crate anyhow;
extern crate serde;

use tesla_metrics::poller::Poller;

fn main() {
    rocket::ignite().attach(Poller::fairing()).launch();
}
