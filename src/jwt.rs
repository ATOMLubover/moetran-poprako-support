use crate::model::auth::Claims;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, decode, encode};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("System time error: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
}

#[derive(Debug)]
pub struct JwtCodec {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

pub type Result<T> = core::result::Result<T, Error>;

impl JwtCodec {
    pub fn new(secret: String) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn encode_token(&self, user_id: &str, exp_secs: usize) -> Result<String> {
        let start = SystemTime::now();

        let since_the_epoch = start.duration_since(UNIX_EPOCH)?;

        let exp = since_the_epoch.as_secs() as usize + exp_secs;

        let claims = Claims {
            sub: user_id.to_owned(),
            exp,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)?;

        Ok(token)
    }

    pub fn decode_token(&self, token: &str) -> Result<Claims> {
        let token_data: TokenData<Claims> = decode(
            token,
            &self.decoding_key,
            &jsonwebtoken::Validation::default(),
        )?;

        Ok(token_data.claims)
    }
}
