use crate::playback::BASE_URL;
use dioxus::prelude::*;
use serde::Serialize;

pub async fn api_post(
    end_point: &str,
    payload: &impl Serialize,
) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();

    let res = client
        .post(format!("http://{BASE_URL}/{end_point}"))
        .json(payload)
        .send()
        .await;

    if let Err(ref e) = res {
        error!("attempt to send api call to endpint: {end_point}, resulted in error: {e}");
    }

    res
}
