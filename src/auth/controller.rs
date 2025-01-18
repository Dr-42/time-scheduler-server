use std::path::Path;

use rsa::{RsaPrivateKey, RsaPublicKey};

use crate::{
    err::{Error, ErrorType},
    err_from_type, err_with_context,
};

use super::handlers::{LoginRequest, LoginResponse};

struct KeyPair {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
}

fn get_key_pair(data_dir: &Path) -> Result<KeyPair, Error> {
    let private_key_path = data_dir.join("private_key.pem");
    let public_key_path = data_dir.join("public_key.pem");

    if !private_key_path.exists() || !public_key_path.exists() {
        todo!()
    } else {
        todo!()
    }
}

pub async fn get_public_key(data_dir: &Path) -> Result<[u8; 512], Error> {
    Err(err_from_type!(ErrorType::NotImplemented))
}

pub async fn get_private_key(data_dir: &Path) -> Result<[u8; 512], Error> {
    todo!()
}

fn decrypt_registration_access_request(
    data_dir: &Path,
    registration_access_token: &str,
) -> Result<String, Error> {
    todo!()
}

pub async fn generate_access_token(device_public_key: &[u8]) -> Result<String, Error> {
    todo!()
}

pub async fn get_refresh_token(device_public_key: &[u8]) -> Result<String, Error> {
    todo!()
}

pub async fn validate_login_request(
    data_dir: &Path,
    login_request: &LoginRequest,
) -> Result<bool, Error> {
    todo!()
}

pub async fn validate_data(data_dir: &Path, jwt: &str) -> Result<LoginResponse, Error> {
    todo!()
}

pub async fn verify_token(data_dir: &Path, jwt: &str) -> Result<bool, Error> {
    todo!()
}
