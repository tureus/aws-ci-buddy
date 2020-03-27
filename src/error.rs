use base64::DecodeError;
use rusoto_core::RusotoError;

#[derive(Debug)]
pub enum EcrDockerLoginError {
    RusotoError(String),
    Base64DecodeError,
    Utf8Error,
    IOError(String),
    UrlParse,
    Other(String),
}

impl<E> From<RusotoError<E>> for EcrDockerLoginError
where
    E: std::error::Error,
{
    fn from(e: RusotoError<E>) -> Self {
        EcrDockerLoginError::RusotoError(format!("{:?}", e))
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
    fn from(e: std::io::Error) -> Self {
        EcrDockerLoginError::IOError(format!("{:?}", e))
    }
}

impl From<url::ParseError> for EcrDockerLoginError {
    fn from(_: url::ParseError) -> Self {
        EcrDockerLoginError::UrlParse
    }
}
