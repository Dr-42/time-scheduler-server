use crate::err::{Error, ErrorType};
use crate::err_from_type;

pub async fn double_decrypt<T>(key1: &[u8], key2: &[u8], data: Vec<u8>) -> Result<T, Error>
where
    T: for<'de> serde::Deserialize<'de>,
{
    todo!()
}
