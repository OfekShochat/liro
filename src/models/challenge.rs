use crate::db::{self, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

const HOSTNAME: &str = "http://localhost:8000";
const CLIENT_ID: &str = "liro-bot-test";

#[derive(Serialize, Deserialize, Debug)]
pub struct Challenge {
    id: u64,
    discord_id: u64,
    code_verifier: Vec<u8>,
}

impl Challenge {
    fn key(id: u64) -> String {
        format!("challenges:{}", id)
    }

    pub async fn new(pool: &db::Pool, discord_id: u64) -> Result<Challenge> {
        let challenge = Self {
            id: rand::random(),
            discord_id,
            code_verifier: pkce::code_verifier(128),
        };

        challenge.save(pool).await?;

        Ok(challenge)
    }

    async fn save(&self, pool: &db::Pool) -> Result<()> {
        let serialized = serde_json::to_string(self)?;
        db::set(pool, &Challenge::key(self.id), &serialized).await?;

        Ok(())
    }

    pub async fn find(pool: &db::Pool, id: u64) -> Result<Option<Challenge>> {
        let serialized = db::get(pool, &Challenge::key(id)).await?;
        Ok(Some(serde_json::from_str(&serialized)?))
    }

    pub fn link(&self) -> String {
        format!("{}/connect/lichess/{}", HOSTNAME, self.id)
    }

    fn code_challenge(&self) -> String {
        pkce::code_challenge(&self.code_verifier)
    }

    fn state(&self) -> String {
        format!("{}", self.id)
    }

    pub fn discord_id(&self) -> u64 {
        self.discord_id
    }

    pub fn lichess_url(&self) -> String {
        let redirect_uri = format!("{}/oauth/callback", HOSTNAME);
        let url = format!(
            "https://lichess.org/oauth\
             ?response_type=code\
             &redirect_uri={}\
             &client_id={}\
             &code_challenge_method=S256\
             &code_challenge={}\
             &state={}",
            redirect_uri,
            CLIENT_ID,
            self.code_challenge(),
            self.state()
        );

        url
    }

    pub fn code_verifier(&self) -> String {
        match std::str::from_utf8(&self.code_verifier) {
            Ok(v) => v.to_string(),
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        }
    }
}

impl fmt::Display for Challenge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Challenge<id={}, user_id={}>", self.id, self.discord_id)
    }
}
