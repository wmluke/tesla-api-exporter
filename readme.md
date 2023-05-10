# tesla-api-exporter

A prometheus exporter for the Tesla vehicle API.

## Usage

```shell
cp .env.example .env
cargo run
```
## Exported Metrics
     
* tesla_charge_state_battery_level
* tesla_charge_state_battery_range
* tesla_charge_state_est_battery_range
* tesla_charge_state_ideal_battery_range
* tesla_charge_state_charge_rate
* tesla_charge_state_minutes_to_full_charge
* tesla_charge_state_charger_voltage
* tesla_charge_state_charger_power
* tesla_charge_state_charger_actual_current
* tesla_drive_state_speed
* tesla_drive_state_power
* tesla_vehicle_state_odometer
* tesla_climate_state_inside_temp
* tesla_climate_state_outside_temp
* tesla_climate_state_driver_temp_setting
* tesla_climate_state_passenger_temp_setting
* tesla_drive_state_latitude
* tesla_drive_state_longitude
* tesla_drive_state_heading
* tesla_car_state
* tesla_is_online
* tesla_drive_state_shift_state

## Auth Tokens

There are multiple apps available to securely generate access tokens yourself, for example:

* [Auth app for Tesla (iOS, macOS)](https://apps.apple.com/us/app/auth-app-for-tesla/id1552058613)
* [Tesla Tokens (Android)](https://play.google.com/store/apps/details?id=net.leveugle.teslatokens)
* [Tesla Auth (macOS, Linux, Windows)](https://github.com/adriankumpf/tesla_auth)
