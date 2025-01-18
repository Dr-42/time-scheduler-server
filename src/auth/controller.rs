use jsonwebtoken::{decode, DecodingKey};

use crate::err::Error;

use super::{
    handlers::{Claims, LoginRequest},
    middleware::TokenState,
};

pub fn verify_user(login_info: &LoginRequest, password_hash: &str) -> bool {
    login_info.key == password_hash
}

pub fn verify_token(token: &str, password_hash: &str) -> Result<TokenState, Error> {
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(password_hash.as_bytes()),
        &Default::default(),
    ) {
        Ok(_) => Ok(TokenState::Valid),
        Err(e) => match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => Ok(TokenState::Expired),
            _ => Ok(TokenState::Unauthorized),
        },
    }
}
