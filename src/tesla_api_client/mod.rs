use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use reqwest::{header, StatusCode};
use reqwest::blocking::Client;

use crate::tesla_api_client::dtos::{
    AuthToken, ErrorReply, Reply, TeslaApiError, Vehicle, VehicleData,
};

pub mod dtos;

static API_URL: &str = "https://owner-api.teslamotors.com";
static CLIENT_ID: &str = "81527cff06843c8634fdc09e8ac0abefb46ac849f38fe1e431c2ef2106796384";
static CLIENT_SECRET: &str = "c7257eb71a564034f9419ee651c7d0e5f7aa6bfbd18bafb5c5c033b093bb2fa3";
static USER_AGENT: &str = "tesla-metrics";

fn create_authenticated_client(access_token: &str) -> Result<Client> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static(USER_AGENT),
    );
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap(),
    );

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .connection_verbose(true)
        .build()?;
    Ok(client)
}

#[derive(Debug, Clone)]
pub struct TeslaApiClient {
    client: Client,
    auth_token: AuthToken,
}

impl TeslaApiClient {
    pub fn authenticate(email: &str, password: &str) -> Result<TeslaApiClient> {
        let mut map = HashMap::new();
        map.insert("grant_type", "password");
        map.insert("client_id", CLIENT_ID);
        map.insert("client_secret", CLIENT_SECRET);
        map.insert("email", email);
        map.insert("password", password);

        let auth_client = reqwest::blocking::Client::new();
        let api_url = &format!(
            "{api_url}/oauth/token?grant_type=password",
            api_url = API_URL
        );
        let response = auth_client
            .post(api_url)
            .header(
                header::USER_AGENT,
                header::HeaderValue::from_static(USER_AGENT),
            )
            .json(&map)
            .send()?;

        let status = &response.status();

        if status == &StatusCode::UNAUTHORIZED {
            return Err(TeslaApiError::LoginFailure.into());
        }

        if !status.is_success() {
            let text: String = response.text()?;
            let error_reply: ErrorReply = serde_json::from_str(&text)?;
            // let error_reply = &response.json::<ErrorReply>()?;
            return Err(TeslaApiError::from(error_reply).into());
        }

        let auth_token = response
            .json::<AuthToken>()?;

        let client = create_authenticated_client(&auth_token.access_token)?;

        Ok(TeslaApiClient { client, auth_token })
    }

    pub fn refresh_auth(&mut self) -> anyhow::Result<()> {
        let mut map = HashMap::new();
        map.insert("grant_type", "refresh_token");
        map.insert("client_id", CLIENT_ID);
        map.insert("client_secret", CLIENT_SECRET);
        map.insert("refresh_token", &self.auth_token.refresh_token);

        let auth_client = reqwest::blocking::Client::new();
        let api_url = &format!(
            "{api_url}/oauth/token?grant_type=refresh_token",
            api_url = API_URL
        );
        let response = auth_client
            .post(api_url)
            .header(
                header::USER_AGENT,
                header::HeaderValue::from_static(USER_AGENT),
            )
            .json(&map)
            .send()?;

        let status = &response.status();
        if !status.is_success() {
            let error_reply = response.json::<ErrorReply>()?;
            return Err(TeslaApiError::from(error_reply).into());
        }

        self.auth_token = response
            .json::<AuthToken>()?;

        self.client = create_authenticated_client(&self.auth_token.access_token)?;
        Ok(())
    }

    pub fn fetch_vehicles(&self) -> anyhow::Result<Vec<Vehicle>> {
        let api_url = format!("{api_url}/api/1/vehicles", api_url = API_URL);
        let response = self
            .client
            .get(&api_url)
            .send()?;

        let status = &response.status();
        if !status.is_success() {
            let error_reply = response.json::<ErrorReply>()?;
            return Err(TeslaApiError::from(error_reply).into());
        }

        let reply = response
            .json::<Reply<Vec<Vehicle>>>()?;

        Ok(reply.response)
    }

    pub fn fetch_vehicle_data(&self, vehicle_id: &i64) -> anyhow::Result<VehicleData> {
        let api_url = format!(
            "{api_url}/api/1/vehicles/{id}/vehicle_data",
            api_url = API_URL,
            id = vehicle_id
        );

        let response = self.client.get(&api_url).send()?;

        let status = &response.status();

        if status == &StatusCode::UNAUTHORIZED {
            return Err(TeslaApiError::LoginFailure.into());
        }

        if !status.is_success() {
            let error_reply = response.json::<ErrorReply>()?;
            return Err(TeslaApiError::from(error_reply).into());
        }

        let reply = response.json::<Reply<VehicleData>>()?;

        Ok(reply.response)
    }

    pub fn wake_vehicle(&self, vehicle_id: &i64) -> anyhow::Result<Vehicle> {
        let api_url = format!(
            "{api_url}/api/1/vehicles/{id}/wake_up",
            api_url = API_URL,
            id = vehicle_id
        );

        let response = self.client.post(&api_url).send()?;

        let status = &response.status();
        if !status.is_success() {
            let error_reply = response.json::<ErrorReply>()?;
            return Err(TeslaApiError::from(error_reply).into());
        }

        let vehicle = response.json::<Reply<Vehicle>>()?.response;
        Ok(vehicle)
    }

    pub fn wake_vehicle_poll(&self, vehicle_id: &i64) -> anyhow::Result<Vehicle> {
        let mut vehicle = self.wake_vehicle(vehicle_id)?;
        let mut count = 0;
        while vehicle.is_asleep() && count < 6 {
            sleep(Duration::from_secs(5));
            vehicle = self.wake_vehicle(vehicle_id)?;
            count += 1;
        }
        if vehicle.is_asleep() {
            return Err(TeslaApiError::WakeTimeout().into());
        }
        Ok(vehicle)
    }

    pub fn fetch_all_vehicles_data(&self) -> anyhow::Result<Vec<VehicleData>> {
        Ok(self
            .fetch_vehicles()?
            .into_iter()
            .filter_map(|v| {
                if v.is_asleep() {
                    self.wake_vehicle_poll(&v.id);
                }
                self.fetch_vehicle_data(&v.id).ok()
            })
            .collect::<Vec<VehicleData>>())
    }
}
