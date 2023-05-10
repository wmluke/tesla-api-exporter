use anyhow::Result;
use dotenv::dotenv;

use tesla_metrics::tesla_api_client::{TeslaApiClient};
use tesla_metrics::tesla_api_client::dtos::AuthToken;

#[test]
fn should_authenticate_and_refresh_authentication() -> Result<()> {
    dotenv().ok();

    let auth_result =
        TeslaApiClient::create(AuthToken::from_env());

    assert_eq!(auth_result.is_ok(), true);

    let mut client = auth_result.unwrap();

    let refresh_result = client.refresh_auth();

    assert_eq!(refresh_result.is_ok(), true);

    Ok(())
}

// #[test]
// fn should_fail_to_authenticate() -> Result<()> {
//     let result = TeslaApiClient::authenticate(Auth {
//         email: "foo@bar.com".to_string(),
//         password: "1234".to_string(),
//     });
//     assert_eq!(result.is_err(), true);
//     assert_eq!(
//         result.unwrap_err().to_string(),
//         "Failed to login"
//     );
//     Ok(())
// }

#[test]
fn should_fetch_vehicles() -> Result<()> {
    dotenv().ok();

    let client = TeslaApiClient::create(AuthToken::from_env())?;

    let vehicles = client.fetch_vehicles()?;

    assert_eq!(vehicles.is_empty(), false);

    Ok(())
}

#[test]
fn should_fail_to_fetch_vehicle_data_bc_vehicle_is_unavailable() -> Result<()> {
    dotenv().ok();

    let client = TeslaApiClient::create(AuthToken::from_env())?;

    let vehicles = client.fetch_vehicles()?;

    assert_eq!(vehicles.is_empty(), false);

    let vehicle = vehicles.get(0).unwrap();
    let vehicle_data_result = client.fetch_vehicle_data(&vehicle.id);

    if vehicle.is_online() {
        assert_eq!(vehicle_data_result?.state, "online");
    } else {
        assert_eq!(vehicle_data_result.is_err(), true);
        assert_eq!(
            vehicle_data_result.unwrap_err().to_string(),
            "Request Timeout vehicle unavailable"
        );
    }

    Ok(())
}

#[test]
fn should_wake_up_the_vehicle() -> Result<()> {
    dotenv().ok();

    let client = TeslaApiClient::create(AuthToken::from_env())?;

    let vehicles = client.fetch_vehicles()?;

    assert_eq!(vehicles.is_empty(), false);

    let vehicle = vehicles.get(0).unwrap();

    let is_online = client.wake_vehicle_poll(&vehicle.id);

    assert_eq!(is_online.is_ok(), true);

    Ok(())
}

#[test]
fn should_fetch_all_vehicle_data() -> Result<()> {
    dotenv().ok();

    let client = TeslaApiClient::create(AuthToken::from_env())?;

    let vehicles_data = client.fetch_all_vehicles_data()?;

    assert_eq!(vehicles_data.is_empty(), false);

    Ok(())
}
