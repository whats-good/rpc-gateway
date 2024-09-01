use std::ops::Deref;

use thiserror::Error;
use url::Url;

use crate::chain::Chain;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetEndpointBase {
    pub name: String,
    pub chain: &'static Chain,
    pub url: Url, // TODO: make this a secret
}

#[derive(Debug, PartialEq, Eq)]
pub struct HttpTargetEndpoint(TargetEndpointBase);

impl TryFrom<TargetEndpointBase> for HttpTargetEndpoint {
    type Error = InitTargetEndpointError;

    fn try_from(value: TargetEndpointBase) -> Result<Self, Self::Error> {
        let scheme = value.url.scheme();
        match scheme {
            "http" | "https" => Ok(HttpTargetEndpoint(value)),
            _ => Err(InitTargetEndpointError::WrongScheme),
        }
    }
}

impl Deref for HttpTargetEndpoint {
    type Target = TargetEndpointBase;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Error, Debug)]
pub enum InitTargetEndpointError {
    #[error("wrong scheme")]
    WrongScheme,
}

#[derive(Debug, PartialEq, Eq)]
pub struct WebSocketTargetEndpoint(TargetEndpointBase);

impl TryFrom<TargetEndpointBase> for WebSocketTargetEndpoint {
    type Error = InitTargetEndpointError;

    fn try_from(value: TargetEndpointBase) -> Result<Self, Self::Error> {
        let scheme = value.url.scheme();
        match scheme {
            "ws" | "wss" => Ok(WebSocketTargetEndpoint(value)),
            _ => Err(InitTargetEndpointError::WrongScheme),
        }
    }
}

#[derive(Debug)]
pub enum TargetEndpoint {
    Http(HttpTargetEndpoint),
    WebSocket(WebSocketTargetEndpoint),
}

impl TryFrom<TargetEndpointBase> for TargetEndpoint {
    type Error = InitTargetEndpointError;

    fn try_from(value: TargetEndpointBase) -> Result<Self, Self::Error> {
        let scheme = value.url.scheme();
        match scheme {
            "http" | "https" => HttpTargetEndpoint::try_from(value).map(TargetEndpoint::Http),
            "ws" | "wss" => WebSocketTargetEndpoint::try_from(value).map(TargetEndpoint::WebSocket),
            _ => Err(InitTargetEndpointError::WrongScheme),
        }
    }
}

impl Deref for TargetEndpoint {
    type Target = TargetEndpointBase;

    fn deref(&self) -> &Self::Target {
        match self {
            TargetEndpoint::Http(e) => &e.0,
            TargetEndpoint::WebSocket(e) => &e.0,
        }
    }
}
