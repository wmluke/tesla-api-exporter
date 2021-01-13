use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use log::warn;
use serde::de::DeserializeOwned;
use ureq::{Agent, Error, Error::Status, Request, Response};

use crate::tesla_api_client::dtos::{
    AuthToken, ErrorReply, Reply, TeslaApiError, Vehicle, VehicleData,
};

pub mod dtos;

static API_URL: &str = "https://owner-api.teslamotors.com";
static CLIENT_ID: &str = "81527cff06843c8634fdc09e8ac0abefb46ac849f38fe1e431c2ef2106796384";
static CLIENT_SECRET: &str = "c7257eb71a564034f9419ee651c7d0e5f7aa6bfbd18bafb5c5c033b093bb2fa3";
static USER_AGENT: &str = "tesla-metrics";

#[derive(Debug, Clone)]
pub struct TeslaApiClient {
    agent: Agent,
    auth_token: AuthToken,
}


impl TeslaApiClient {
    pub fn authenticate(email: &str, password: &str) -> Result<TeslaApiClient> {
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();

        let api_url = &format!(
            "{api_url}/oauth/token?grant_type=password",
            api_url = API_URL
        );

        let result = agent
            .post(api_url)
            .set("User-Agent", USER_AGENT,
            )
            .send_json(ureq::json!({
                "grant_type": "password",
                "client_id": CLIENT_ID,
                "client_secret": CLIENT_SECRET,
                "email": email,
                "password": password,
            }));

        let auth_token = TeslaApiClient::handle_result::<AuthToken>(result)?;
        Ok(TeslaApiClient { agent, auth_token })
    }

    pub fn refresh_auth(&mut self) -> anyhow::Result<()> {
        let api_url = &format!(
            "{api_url}/oauth/token?grant_type=refresh_token",
            api_url = API_URL
        );
        let result = self.http_post(api_url)
            .send_json(ureq::json!({
                "grant_type": "refresh_token",
                "client_id": CLIENT_ID,
                "client_secret": CLIENT_SECRET,
                "refresh_token": &self.auth_token.refresh_token,
            }));

        self.auth_token = TeslaApiClient::handle_result::<AuthToken>(result)?;
        Ok(())
    }

    pub fn fetch_vehicle(&self, vehicle_id: &i64) -> anyhow::Result<Vehicle> {
        let api_url = format!("{api_url}/api/1/vehicles/{id}",
                              api_url = API_URL,
                              id = vehicle_id,
        );
        let result = self
            .http_get(&api_url)
            .call();

        let reply = TeslaApiClient::handle_result::<Reply<Vehicle>>(result)?;
        Ok(reply.response)
    }

    pub fn fetch_vehicles(&self) -> anyhow::Result<Vec<Vehicle>> {
        let api_url = format!("{api_url}/api/1/vehicles", api_url = API_URL);
        let result = self
            .http_get(&api_url)
            .call();

        let reply = TeslaApiClient::handle_result::<Reply<Vec<Vehicle>>>(result)?;
        Ok(reply.response)
    }

    pub fn fetch_vehicle_data(&self, vehicle_id: &i64) -> anyhow::Result<VehicleData> {
        let api_url = format!(
            "{api_url}/api/1/vehicles/{id}/vehicle_data",
            api_url = API_URL,
            id = vehicle_id
        );

        let result = self.http_get(&api_url).call();

        let reply = TeslaApiClient::handle_result::<Reply<VehicleData>>(result)?;
        Ok(reply.response)
    }

    fn handle_result<T: DeserializeOwned>(result: Result<Response, Error>) -> Result<T> {
        match result {
            Err(Status(401, _)) => {
                return Err(TeslaApiError::LoginFailure.into());
            }
            Err(Status(_, response)) => {
                let text: String = response.into_string()?;
                let error_reply: ErrorReply = serde_json::from_str(&text)?;
                return Err(TeslaApiError::from(error_reply).into());
            }
            Err(Error::Transport(_)) => {
                return Err(TeslaApiError::Unknown.into());
            }
            Ok(response) => {
                let json: String = response.into_string()?;
                let result: serde_json::error::Result<T> = serde_json::from_str(&json);
                match result {
                    Ok(reply) => Ok(reply),
                    Err(err) => {
                        Err(TeslaApiError::JsonDeserializationError(format!("{:?}: {}", err, json)).into())
                    }
                }
            }
        }
    }

    pub fn wake_vehicle(&self, vehicle_id: &i64) -> anyhow::Result<Vehicle> {
        let api_url = format!(
            "{api_url}/api/1/vehicles/{id}/wake_up",
            api_url = API_URL,
            id = vehicle_id
        );

        let result = self.http_post(&api_url).call();

        let reply = TeslaApiClient::handle_result::<Reply<Vehicle>>(result)?;
        Ok(reply.response)
    }

    pub fn wake_vehicle_poll(&self, vehicle_id: &i64) -> anyhow::Result<()> {
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
        Ok(())
    }

    pub fn fetch_all_vehicles_data(&self) -> anyhow::Result<Vec<VehicleData>> {
        Ok(self
            .fetch_vehicles()?
            .into_iter()
            .filter_map(|v| {
                if v.is_asleep() {
                    if let Err(e) = self.wake_vehicle_poll(&v.id) {
                        warn!("Failed to wake vehicle {:?}", e)
                    }
                }
                self.fetch_vehicle_data(&v.id).ok()
            })
            .collect::<Vec<VehicleData>>())
    }

    fn http_get(&self, url: &String) -> Request {
        self.agent.get(url)
            .set("Authorization", &format!("Bearer {}", &self.auth_token.access_token))
            .set("User-Agent", USER_AGENT)
    }

    fn http_post(&self, url: &String) -> Request {
        self.agent.post(url)
            .set("Authorization", &format!("Bearer {}", &self.auth_token.access_token))
            .set("User-Agent", USER_AGENT)
    }
}
