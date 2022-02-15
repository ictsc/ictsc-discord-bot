use crate::{errors, Result};
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap};
use serenity::http::Http;
use serenity::model::prelude::*;

#[async_trait]
pub trait ProblemRecreater {
    async fn recreate(
        &self,
        team_id: String,
        problem_id: String,
    ) -> Result<String>;
}

pub struct ProblemRecreateManager {
    baseurl: String,
    client: Client,
}

impl ProblemRecreateManager {
    pub fn new(baseurl: String, username: String, password: String) -> Self {
        let secret = base64::encode(format!("{}:{}", username, password));

        let mut headers = HeaderMap::new();
        headers.append(CONTENT_TYPE, "application/x-www-form-urlencoded".parse().unwrap());
        headers.append(AUTHORIZATION, format!("Basic {}", secret).parse().unwrap());

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build().unwrap();

        Self { baseurl, client }
    }
}

#[async_trait]
impl ProblemRecreater for ProblemRecreateManager {
    async fn recreate(&self, team_id: String, problem_id: String) -> Result<String>
    {
        let url = format!("{}/admin/postJob", self.baseurl);

        let response = self.client.post(url)
            .body(format!("team_id={}&prob_id={}", team_id, problem_id))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let data = response.bytes().await?.to_vec();
                Ok(format!("{}{}", self.baseurl, String::from_utf8(data).unwrap()))
            }
            StatusCode::BAD_REQUEST => Err(errors::UserError::RequestInQueue.into()),
            StatusCode::NOT_FOUND => Err(errors::UserError::OutOfCompetitionTime.into()),
            _ => {
                let message = format!("recreate server returns unexpected status code: {:?}", response.status());
                Err(errors::SystemError::UnexpectedError(message).into())
            }
        }
    }
}

