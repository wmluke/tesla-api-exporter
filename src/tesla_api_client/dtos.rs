use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TeslaApiError {
    #[error("Failed to login")]
    LoginFailure,
    #[error("Request Timeout vehicle unavailable")]
    VehicleUnavailable(),
    #[error("Cannot wake vehicle")]
    WakeTimeout(),
    #[error("Unknown Error: {0:?}")]
    Unknown(ErrorReply),
}

impl From<ErrorReply> for TeslaApiError {
    fn from(reply: ErrorReply) -> Self {
        if reply.error.starts_with("vehicle unavailable:") {
            return TeslaApiError::VehicleUnavailable();
        }
        return TeslaApiError::Unknown(reply);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// # `vehicle_id` vs `id`
/// One potentially confusing part of Tesla's API is the switching use of the `id` and `vehicle_id` of
/// the car. The id field is an identifier for the car on the owner-api endpoint. The vehicle_id
/// field is for identifying the car across different endpoints, such as the streaming or Autopark
/// APIs.
///
/// For the state and command APIs, you should be using the id field. If your JSON parser doesn't
/// support large numbers (>32 bit), then you can use the id_s field for a string version of the ID.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vehicle {
    pub id: i64,
    pub id_s: String,
    pub vehicle_id: i64,
    pub vin: String,
    pub display_name: String,
    pub option_codes: String,
    // color: String,
    pub tokens: Vec<String>,
    pub state: String,
    pub calendar_enabled: bool,
    pub api_version: i32,
    // backseat_token: String,
    // backseat_token_updated_at: i64,
}

impl Vehicle {
    pub fn is_online(&self) -> bool {
        self.state.eq("online")
    }

    pub fn is_asleep(&self) -> bool {
        self.state.eq("asleep")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VehicleData {
    pub id: i64,
    pub id_s: String,
    pub user_id: i64,
    pub vehicle_id: i64,
    pub vin: String,
    pub display_name: String,
    pub option_codes: String,
    pub access_type: String,
    pub tokens: Vec<String>,
    pub state: String,
    pub calendar_enabled: bool,
    pub api_version: i32,
    pub drive_state: VehicleDriveState,
    pub climate_state: VehicleClimateState,
    pub charge_state: VehicleChargeState,
    pub gui_settings: VehicleGUIState,
    pub vehicle_state: VehicleState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VehicleDriveState {
    pub gps_as_of: i64,
    pub heading: f32,
    pub latitude: f32,
    pub longitude: f32,
    pub native_latitude: f32,
    pub native_location_supported: i32,
    pub native_longitude: f32,
    pub native_type: String,
    pub power: i32,
    pub shift_state: Option<String>,
    pub speed: Option<f32>,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VehicleClimateState {
    pub battery_heater: bool,
    // pub battery_heater_no_power: null,
    pub climate_keeper_mode: String,
    pub defrost_mode: i32,
    pub driver_temp_setting: f64,
    pub fan_status: i32,
    pub inside_temp: f64,
    pub is_auto_conditioning_on: bool,
    pub is_climate_on: bool,
    pub is_front_defroster_on: bool,
    pub is_preconditioning: bool,
    pub is_rear_defroster_on: bool,
    pub left_temp_direction: f64,
    pub max_avail_temp: f64,
    pub min_avail_temp: f64,
    pub outside_temp: f64,
    pub passenger_temp_setting: f64,
    pub remote_heater_control_enabled: bool,
    pub right_temp_direction: f64,
    pub seat_heater_left: i32,
    pub seat_heater_right: i32,
    pub side_mirror_heaters: bool,
    pub timestamp: i64,
    pub wiper_blade_heater: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VehicleChargeState {
    pub battery_heater_on: bool,
    pub battery_level: i32,
    pub battery_range: f64,
    pub charge_current_request: i32,
    pub charge_current_request_max: i32,
    pub charge_enable_request: bool,
    pub charge_energy_added: f64,
    pub charge_limit_soc: i32,
    pub charge_limit_soc_max: i32,
    pub charge_limit_soc_min: i32,
    pub charge_limit_soc_std: i32,
    pub charge_miles_added_ideal: f64,
    pub charge_miles_added_rated: f64,
    // pub charge_port_cold_weather_mode: Option<String>,
    pub charge_port_door_open: bool,
    pub charge_port_latch: String,
    pub charge_rate: f64,
    pub charge_to_max_range: bool,
    pub charger_actual_current: i32,
    // pub charger_phases: i32,
    pub charger_pilot_current: i32,
    pub charger_power: i32,
    pub charger_voltage: i32,
    pub charging_state: String,
    pub conn_charge_cable: String,
    pub est_battery_range: f64,
    pub fast_charger_brand: String,
    pub fast_charger_present: bool,
    pub fast_charger_type: String,
    pub ideal_battery_range: f64,
    pub managed_charging_active: bool,
    managed_charging_start_time: Option<i64>,
    pub managed_charging_user_canceled: bool,
    pub max_range_charge_counter: i32,
    pub minutes_to_full_charge: i32,
    // pub not_enough_power_to_heat: null,
    pub scheduled_charging_pending: bool,
    scheduled_charging_start_time: Option<i64>,
    pub time_to_full_charge: f64,
    pub timestamp: i64,
    pub trip_charging: bool,
    pub usable_battery_level: i32,
    // user_charge_enable_request: null
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VehicleGUIState {
    pub gui_24_hour_time: bool,
    pub gui_charge_rate_units: String,
    pub gui_distance_units: String,
    pub gui_range_display: String,
    pub gui_temperature_units: String,
    pub show_range_units: bool,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VehicleState {
    pub api_version: i32,
    // pub autopark_state_v2: String,
    // pub autopark_style: String,
    pub calendar_supported: bool,
    pub car_version: String,
    pub center_display_state: i32,
    pub df: i32,
    pub dr: i32,
    pub fd_window: i32,
    pub fp_window: i32,
    pub ft: i32,
    // pub homelink_device_count: i32,
    // pub homelink_nearby: bool,
    pub is_user_present: bool,
    // pub last_autopark_error: String,
    pub locked: bool,
    pub media_state: VehicleMediaState,
    pub notifications_supported: bool,
    pub odometer: f64,
    pub parsed_calendar_supported: bool,
    pub pf: i32,
    pub pr: i32,
    pub rd_window: i32,
    pub remote_start: bool,
    pub remote_start_enabled: bool,
    pub remote_start_supported: bool,
    pub rp_window: i32,
    pub rt: i32,
    pub sentry_mode: bool,
    pub sentry_mode_available: bool,
    #[serde(default)]
    pub smart_summon_available: bool,
    #[serde(default)]
    pub summon_standby_mode_enabled: bool,
    #[serde(default)]
    pub sun_roof_percent_open: i32,
    #[serde(default)]
    pub sun_roof_state: String,
    pub timestamp: i64,
    pub valet_mode: bool,
    #[serde(default)]
    pub valet_pin_needed: bool,
    pub vehicle_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VehicleMediaState {
    pub remote_control_enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VehicleSoftwareState {
    pub download_perc: i32,
    pub expected_duration_sec: i64,
    pub install_perc: i32,
    pub status: String,
    pub version: String,
}

pub struct VehicleSpeedLimitModeState {
    pub active: bool,
    pub current_limit_mph: i64,
    pub max_limit_mph: i32,
    pub min_limit_mph: i32,
    pub pin_code_set: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VehicleConfig {
    pub can_accept_navigation_requests: bool,
    pub can_actuate_trunks: bool,
    pub car_special_type: String,
    pub car_type: String,
    pub charge_port_type: String,
    pub default_charge_to_max: bool,
    pub ece_restrictions: bool,
    pub eu_vehicle: bool,
    pub exterior_color: String,
    pub has_air_suspension: bool,
    pub has_ludicrous_mode: bool,
    pub motorized_charge_port: bool,
    pub plg: bool,
    pub rear_seat_heaters: i32,
    // pub rear_seat_type: i32,
    pub rhd: bool,
    pub roof_color: String,
    // pub seat_type: i32,
    pub spoiler_type: String,
    // pub sun_roof_installed: i32,
    pub third_row_seats: String,
    pub timestamp: i64,
    pub trim_badging: String,
    pub use_range_badging: bool,
    pub wheel_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reply<T> {
    pub response: T,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ErrorReply {
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub error_description: String,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn should_deserialize_to_vehicle_data() -> Result<()> {
        let json = r#"
        {
          "id": 41614331478102467,
          "user_id": 769546,
          "vehicle_id": 1687424833,
          "vin": "5YJ3E1EA4KF311487",
          "display_name": "Bellwood Auto",
          "option_codes": "AD15,MDL3,PBSB,RENA,BT37,ID3W,RF3G,S3PB,DRLH,DV2W,W39B,APF0,COUS,BC3B,CH07,PC30,FC3P,FG31,GLFR,HL31,HM31,IL31,LTPB,MR31,FM3B,RS3H,SA3P,STCP,SC04,SU3C,T3CA,TW00,TM00,UT3P,WR00,AU3P,APH3,AF00,ZCST,MI00,CDM0",
          "color": null,
          "access_type": "OWNER",
          "tokens": [
            "dd4cfa67bb06841c",
            "00a820db0b3aae04"
          ],
          "state": "online",
          "in_service": false,
          "id_s": "41614331478102467",
          "calendar_enabled": true,
          "api_version": 14,
          "backseat_token": null,
          "backseat_token_updated_at": null,
          "vehicle_config": {
            "can_accept_navigation_requests": true,
            "can_actuate_trunks": true,
            "car_special_type": "base",
            "car_type": "model3",
            "charge_port_type": "US",
            "default_charge_to_max": false,
            "ece_restrictions": false,
            "eu_vehicle": false,
            "exterior_color": "DeepBlue",
            "exterior_trim": "Chrome",
            "has_air_suspension": false,
            "has_ludicrous_mode": false,
            "key_version": 2,
            "motorized_charge_port": true,
            "plg": false,
            "rear_seat_heaters": 0,
            "rear_seat_type": null,
            "rhd": false,
            "roof_color": "Glass",
            "seat_type": null,
            "spoiler_type": "None",
            "sun_roof_installed": null,
            "third_row_seats": "<invalid>",
            "timestamp": 1609734298988,
            "use_range_badging": true,
            "wheel_type": "Stiletto19"
          },
          "charge_state": {
            "battery_heater_on": false,
            "battery_level": 87,
            "battery_range": 208.15,
            "charge_current_request": 32,
            "charge_current_request_max": 32,
            "charge_enable_request": true,
            "charge_energy_added": 30.11,
            "charge_limit_soc": 90,
            "charge_limit_soc_max": 100,
            "charge_limit_soc_min": 50,
            "charge_limit_soc_std": 90,
            "charge_miles_added_ideal": 137.5,
            "charge_miles_added_rated": 137.5,
            "charge_port_cold_weather_mode": false,
            "charge_port_door_open": false,
            "charge_port_latch": "Engaged",
            "charge_rate": 0.0,
            "charge_to_max_range": false,
            "charger_actual_current": 0,
            "charger_phases": null,
            "charger_pilot_current": 32,
            "charger_power": 0,
            "charger_voltage": 2,
            "charging_state": "Disconnected",
            "conn_charge_cable": "<invalid>",
            "est_battery_range": 153.79,
            "fast_charger_brand": "<invalid>",
            "fast_charger_present": false,
            "fast_charger_type": "<invalid>",
            "ideal_battery_range": 208.15,
            "managed_charging_active": false,
            "managed_charging_start_time": null,
            "managed_charging_user_canceled": false,
            "max_range_charge_counter": 0,
            "minutes_to_full_charge": 0,
            "not_enough_power_to_heat": null,
            "scheduled_charging_pending": false,
            "scheduled_charging_start_time": null,
            "time_to_full_charge": 0.0,
            "timestamp": 1609734298988,
            "trip_charging": false,
            "usable_battery_level": 87,
            "user_charge_enable_request": null
          },
          "climate_state": {
            "battery_heater": false,
            "battery_heater_no_power": null,
            "climate_keeper_mode": "off",
            "defrost_mode": 0,
            "driver_temp_setting": 21.7,
            "fan_status": 0,
            "inside_temp": 11.0,
            "is_auto_conditioning_on": false,
            "is_climate_on": false,
            "is_front_defroster_on": false,
            "is_preconditioning": false,
            "is_rear_defroster_on": false,
            "left_temp_direction": 830,
            "max_avail_temp": 28.0,
            "min_avail_temp": 15.0,
            "outside_temp": 11.0,
            "passenger_temp_setting": 21.7,
            "remote_heater_control_enabled": false,
            "right_temp_direction": 830,
            "seat_heater_left": 0,
            "seat_heater_right": 0,
            "side_mirror_heaters": false,
            "timestamp": 1609734298988,
            "wiper_blade_heater": false
          },
          "drive_state": {
            "gps_as_of": 1609733536,
            "heading": 284,
            "latitude": 41.097174,
            "longitude": -73.770422,
            "native_latitude": 41.097174,
            "native_location_supported": 1,
            "native_longitude": -73.770422,
            "native_type": "wgs",
            "power": 0,
            "shift_state": null,
            "speed": null,
            "timestamp": 1609734298988
          },
          "gui_settings": {
            "gui_24_hour_time": false,
            "gui_charge_rate_units": "mi/hr",
            "gui_distance_units": "mi/hr",
            "gui_range_display": "Rated",
            "gui_temperature_units": "F",
            "show_range_units": false,
            "timestamp": 1609734298988
          },
          "vehicle_state": {
            "api_version": 14,
            "autopark_state_v3": "unavailable",
            "calendar_supported": true,
            "car_version": "2020.48.26 e3178ea250ba",
            "center_display_state": 0,
            "df": 0,
            "dr": 0,
            "fd_window": 0,
            "fp_window": 0,
            "ft": 0,
            "is_user_present": false,
            "locked": true,
            "media_state": {
              "remote_control_enabled": true
            },
            "notifications_supported": true,
            "odometer": 7469.486058,
            "parsed_calendar_supported": true,
            "pf": 0,
            "pr": 0,
            "rd_window": 0,
            "remote_start": false,
            "remote_start_enabled": true,
            "remote_start_supported": true,
            "rp_window": 0,
            "rt": 0,
            "sentry_mode": false,
            "sentry_mode_available": true,
            "software_update": {
              "download_perc": 0,
              "expected_duration_sec": 2700,
              "install_perc": 1,
              "status": "",
              "version": "2020.48.26"
            },
            "speed_limit_mode": {
              "active": false,
              "current_limit_mph": 85.0,
              "max_limit_mph": 90,
              "min_limit_mph": 50,
              "pin_code_set": false
            },
            "timestamp": 1609734298988,
            "valet_mode": false,
            "vehicle_name": "Bellwood Auto"
          }
        }
        "#;

        let vehicle_data: VehicleData = serde_json::from_str(json)?;

        assert_eq!(vehicle_data.id, 41614331478102467);

        Ok(())
    }
}
