use std::path::Path;

use crate::err::Error;

pub async fn get_public_key(data_dir: &Path) -> Result<Vec<u8>, Error> {
    todo!()
}

pub async fn get_token(data_dir: &Path, data_stream: Vec<u8>) -> Result<String, Error> {
    todo!()
}

pub async fn verify_token(data_dir: &Path, token: &str) -> Result<bool, Error> {
    todo!()
}
