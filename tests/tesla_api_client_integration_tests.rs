#[macro_use]
extern crate dotenv_codegen;

use anyhow::Result;

use tesla_metrics::tesla_api_client::TeslaApiClient;

#[test]
fn should_authenticate_and_refresh_authentication() -> Result<()> {
    let auth_result =
        TeslaApiClient::authenticate(dotenv!("TESLA_EMAIL"), dotenv!("TESLA_PASSWORD"));

    assert_eq!(auth_result.is_ok(), true);

    let mut client = auth_result.unwrap();

    let refresh_result = client.refresh_auth();

    assert_eq!(refresh_result.is_ok(), true);

    Ok(())
}

#[test]
fn should_fail_to_authenticate() -> Result<()> {
    let result = TeslaApiClient::authenticate("foo@bar.com", "1234");
    assert_eq!(result.is_err(), true);
    assert_eq!(
        result.unwrap_err().to_string(),
        "Failed to login"
    );
    Ok(())
}

#[test]
fn should_fetch_vehicles() -> Result<()> {
    let client = TeslaApiClient::authenticate(dotenv!("TESLA_EMAIL"), dotenv!("TESLA_PASSWORD"))?;

    let vehicles = client.fetch_vehicles()?;

    assert_eq!(vehicles.is_empty(), false);

    Ok(())
}

#[test]
fn should_fail_to_fetch_vehicle_data_bc_vehicle_is_unavailable() -> Result<()> {
    let client = TeslaApiClient::authenticate(dotenv!("TESLA_EMAIL"), dotenv!("TESLA_PASSWORD"))?;

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
    let client = TeslaApiClient::authenticate(dotenv!("TESLA_EMAIL"), dotenv!("TESLA_PASSWORD"))?;

    let vehicles = client.fetch_vehicles()?;

    assert_eq!(vehicles.is_empty(), false);

    let vehicle = vehicles.get(0).unwrap();

    let is_online = client.wake_vehicle_poll(&vehicle.id);

    assert_eq!(is_online.is_ok(), true);

    Ok(())
}

#[test]
fn should_fetch_all_vehicle_data() -> Result<()> {
    let client = TeslaApiClient::authenticate(dotenv!("TESLA_EMAIL"), dotenv!("TESLA_PASSWORD"))?;

    let vehicles_data = client.fetch_all_vehicles_data()?;

    assert_eq!(vehicles_data.is_empty(), false);

    Ok(())
}
