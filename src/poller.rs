use core::fmt;
use std::fmt::Display;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;

use anyhow::Result;
use log::{error, info, warn};
use once_cell::sync::Lazy;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::Rocket;
use rocket_prometheus::{
    prometheus::{IntGaugeVec, opts},
    PrometheusMetrics,
};
use rocket_prometheus::prometheus::GaugeVec;
use serde::export::Formatter;

use crate::tesla_api_client::dtos::{Vehicle, VehicleData};
use crate::tesla_api_client::TeslaApiClient;

static BATTERY_LEVEL_GAUGE: Lazy<IntGaugeVec> = Lazy::new(|| {
    IntGaugeVec::new(opts!("tesla_charge_state_battery_level", "Battery Level (%)"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static BATTERY_RANGE_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_charge_state_battery_range", "Battery Range (Miles)"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static BATTERY_EST_RANGE_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_charge_state_est_battery_range", "Estimated Battery Range (Miles)"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static BATTERY_IDEAL_RANGE_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_charge_state_ideal_battery_range", "Ideal Battery Range (Miles)"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static CHARGE_RATE_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_charge_state_charge_rate", "Battery Charge Rate"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static TIME_TO_FULL_CHARGE_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_charge_state_minutes_to_full_charge", "Time to Full Charge"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static CHARGER_VOLTAGE_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_charge_state_charger_voltage", "Charger Voltage"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static CHARGER_POWER_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_charge_state_charger_power", "Charger Power"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static CHARGER_ACTUAL_CURRENT_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_charge_state_charger_actual_current", "Charger Actual Current"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static SPEED_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_drive_state_speed", "Vehicle speed (MPH)"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static POWER_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_drive_state_power", "Vehicle Power"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static ODOMETER_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_vehicle_state_odometer", "Vehicle odometer (Miles)"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static INSIDE_TEMPERATURE_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_climate_state_inside_temp", "Inside Temperature (DegC)"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static OUTSIDE_TEMPERATURE_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_climate_state_outside_temp", "Outside Temperature (DegC)"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static DRIVER_TEMPERATURE_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_climate_state_driver_temp_setting", "Driver's Temperature Setting (DegC)"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static PASSENGER_TEMPERATURE_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_climate_state_passenger_temp_setting", "Passenger's Temperature Setting (DegC)"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static GEO_LAT_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_drive_state_latitude", "Vehicle Latitude"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static GEO_LONG_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_drive_state_longitude", "Vehicle Longitude"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

static GEO_HEADING_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    GaugeVec::new(opts!("tesla_drive_state_heading", "Vehicle Heading"), &["car_name", "car_state"])
        .expect("Could not create lazy GaugeVec")
});

#[derive(Debug)]
pub enum CarState<'a> {
    Parked(&'a VehicleData),
    Charging(&'a VehicleData),
    Driving(&'a VehicleData),
}

impl<'a> CarState<'a> {
    pub fn wait(&self) -> Duration {
        match self {
            CarState::Parked(_) => {
                Duration::from_secs(15 * 60)
            }
            CarState::Charging(v) => {
                if v.charge_state.fast_charger_present {
                    Duration::from_secs(5)
                } else {
                    Duration::from_secs(15)
                }
            }
            CarState::Driving(_) => {
                Duration::from_secs(5)
            }
        }
    }
}

impl<'a> Display for CarState<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CarState::Parked(_) => {
                write!(f, "Parked")
            }
            CarState::Charging(_) => {
                write!(f, "Charging")
            }
            CarState::Driving(_) => {
                write!(f, "Driving")
            }
        }
    }
}

impl<'a> From<&'a VehicleData> for CarState<'a> {
    fn from(v: &'a VehicleData) -> Self {
        let speed = v.drive_state.speed.unwrap_or_default();
        let shift = v.drive_state.shift_state.as_deref().unwrap_or_default();
        if shift.eq("R") || shift.eq("D") || shift.eq("N") || speed > 0.0 {
            return CarState::Driving(&v);
        }
        let charging_state = v.charge_state.charging_state.clone();
        if charging_state.eq("Disconnected") {
            return CarState::Parked(&v);
        }
        CarState::Charging(&v)
    }
}


fn collect_vehicle_metrics(client: TeslaApiClient, vehicle: Vehicle, stop: Arc<AtomicBool>) -> Result<()> {
    // TODO: reset error count after some duration
    let mut error_count = 0;

    while !stop.load(Ordering::SeqCst) && error_count < 10 {
        if let Err(err) = client.wake_vehicle_poll(&vehicle.id) {
            warn!("Failed to wake up vehicle: error={:?}", err);
        }

        match client.fetch_vehicle_data(&vehicle.id) {
            Ok(vehicle_data) => {
                let car_state = CarState::from(&vehicle_data);

                BATTERY_LEVEL_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(i64::from(vehicle_data.charge_state.battery_level));

                BATTERY_RANGE_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.charge_state.battery_range);

                BATTERY_EST_RANGE_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.charge_state.est_battery_range);

                BATTERY_IDEAL_RANGE_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.charge_state.ideal_battery_range);

                TIME_TO_FULL_CHARGE_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.charge_state.time_to_full_charge);

                CHARGE_RATE_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.charge_state.charge_rate);

                CHARGER_VOLTAGE_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.charge_state.charger_voltage);

                CHARGER_POWER_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.charge_state.charger_power);

                CHARGER_ACTUAL_CURRENT_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.charge_state.charger_actual_current);

                SPEED_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.drive_state.speed.unwrap_or(0.0_f64));

                POWER_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.drive_state.power);

                ODOMETER_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.vehicle_state.odometer);

                INSIDE_TEMPERATURE_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.climate_state.inside_temp);

                OUTSIDE_TEMPERATURE_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.climate_state.outside_temp);

                DRIVER_TEMPERATURE_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.climate_state.driver_temp_setting);

                PASSENGER_TEMPERATURE_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.climate_state.passenger_temp_setting);

                GEO_LAT_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.drive_state.latitude);

                GEO_LONG_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.drive_state.longitude);

                GEO_HEADING_GAUGE
                    .with_label_values(&[&vehicle_data.display_name, &car_state.to_string()])
                    .set(vehicle_data.drive_state.heading);


                let duration = car_state.wait();
                info!("Collected vehicle metrics: Vehicle=\"{}\" CarState={} Waiting={:?}", &vehicle.display_name, car_state, duration);
                sleep(duration);
            }
            Err(err) => {
                error_count += 1;
                error!("Failed to fetch vehicle data: error_count={} error={:?}", error_count, err);
                sleep(Duration::from_secs(60));
            }
        }
    }
    Ok(())
}

fn start_jobs() -> Result<JobHandles> {
    let client = TeslaApiClient::authenticate(dotenv!("TESLA_EMAIL"), dotenv!("TESLA_PASSWORD"))?;
    let mut handles = JobHandles::default();
    let vehicles = client.fetch_vehicles()?;
    for v in vehicles {
        info!("Started collecting vehicle metrics: Vehicle=\"{}\"", &v.display_name);
        let s = handles.get_stop();
        let c = client.clone();
        handles.add_handle(thread::spawn(move || {
            if let Err(err) = collect_vehicle_metrics(c, v, s) {
                warn!("Failed to collect vehicle metrics: {:?}", err);
            }
        }));
    }
    Ok(handles)
}

pub struct JobHandles {
    stop: Arc<AtomicBool>,
    handles: Vec<JoinHandle<()>>,
}

impl JobHandles {
    pub fn add_handle(&mut self, handle: JoinHandle<()>) {
        self.handles.push(handle);
    }

    pub fn get_stop(&self) -> Arc<AtomicBool> {
        self.stop.clone()
    }
}

impl Default for JobHandles {
    fn default() -> Self {
        JobHandles {
            stop: Arc::new(AtomicBool::new(false)),
            handles: Vec::new(),
        }
    }
}

impl Drop for JobHandles {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        for handle in self.handles.drain(0..) {
            handle.join().unwrap();
        }
    }
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
            .register(Box::new(BATTERY_LEVEL_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(BATTERY_RANGE_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(BATTERY_EST_RANGE_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(BATTERY_IDEAL_RANGE_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(CHARGER_VOLTAGE_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(CHARGER_POWER_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(CHARGER_ACTUAL_CURRENT_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(TIME_TO_FULL_CHARGE_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(CHARGE_RATE_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(SPEED_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(POWER_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(ODOMETER_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(INSIDE_TEMPERATURE_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(OUTSIDE_TEMPERATURE_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(DRIVER_TEMPERATURE_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(PASSENGER_TEMPERATURE_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(GEO_LAT_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(GEO_LONG_GAUGE.clone()))
            .unwrap();

        prometheus
            .registry()
            .register(Box::new(GEO_HEADING_GAUGE.clone()))
            .unwrap();

        Ok(rocket
            .attach(prometheus.clone())
            .mount("/metrics", prometheus)
            .manage(start_jobs().unwrap_or_default()))
    }
}
