use base64::DecodeError;
use rusoto_core::{Region, RusotoError};
use rusoto_ecr::{Ecr, EcrClient, GetAuthorizationTokenRequest};
use std::error::Error;

fn main() -> Result<(), EcrDockerLoginError> {
    let client = EcrClient::new(Region::default());

    let auth_token_resp = client
        .get_authorization_token(GetAuthorizationTokenRequest {
            ..Default::default()
        })
        .sync()?;

    let auth_data = auth_token_resp
        .authorization_data
        .unwrap_or_else(|| panic!("no auth data in the valid token response. weird!"));
    assert_eq!(auth_data.len(), 1);
    let auth_data = &auth_data[0];

    let auth_token = auth_data.authorization_token.as_ref().unwrap();
    let decoded_auth_token_vec = base64::decode(auth_token)?;
    let decoded_auth_token = String::from_utf8(decoded_auth_token_vec)?;
    let mut auth_token_parts = decoded_auth_token.split(":");
    let username = auth_token_parts.next().expect("username part");
    let password = auth_token_parts.next().expect("password part");
    let proxy_endpoint = auth_data.proxy_endpoint.as_ref().expect("proxy endpoint");

    println!(
        "docker login -u {} -p {} {}",
        username, password, proxy_endpoint
    );

    Ok(())
}

#[derive(Debug)]
enum EcrDockerLoginError {
    RusotoError,
    Base64DecodeError,
    Utf8Error,
    IOError,
}

impl<E> From<RusotoError<E>> for EcrDockerLoginError
where
    E: Error,
{
    fn from(_: RusotoError<E>) -> Self {
        EcrDockerLoginError::RusotoError
    }
}

impl From<DecodeError> for EcrDockerLoginError {
    fn from(_: DecodeError) -> Self {
        EcrDockerLoginError::Base64DecodeError
    }
}

impl From<std::string::FromUtf8Error> for EcrDockerLoginError {
    fn from(_: std::string::FromUtf8Error) -> Self {
        EcrDockerLoginError::Utf8Error
    }
}

impl From<std::io::Error> for EcrDockerLoginError {
    fn from(_: std::io::Error) -> Self {
        EcrDockerLoginError::IOError
    }
}
