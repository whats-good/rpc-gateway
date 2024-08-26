use std::ops::Deref;

use derive_more::derive::Constructor;
use secrecy::{CloneableSecret, DebugSecret, ExposeSecret, Secret};
use url::Url;
use zeroize::Zeroize;

use crate::chain::Chain;

#[derive(Clone, Debug)]
pub struct TargetEndpointBase {
    pub name: String,
    pub chain: &'static Chain,
    pub url: Secret<ZeroizableUrl>,
}

#[derive(Debug)]
pub struct HttpTargetEndpoint(TargetEndpointBase);

#[derive(Debug)]
pub enum InitTargetEndpointError {
    WrongScheme,
}

#[derive(Debug)]
pub struct WebSocketTargetEndpoint(TargetEndpointBase);

#[derive(Debug)]
pub enum TargetEndpoint {
    Http(HttpTargetEndpoint),
    WebSocket(WebSocketTargetEndpoint),
}

#[derive(Constructor, Clone)]
pub struct ZeroizableUrl(Url);

impl Zeroize for ZeroizableUrl {
    fn zeroize(&mut self) {
        // TODO: implement
    }
}

impl From<Url> for ZeroizableUrl {
    fn from(value: Url) -> Self {
        ZeroizableUrl(value)
    }
}

impl CloneableSecret for ZeroizableUrl {}
impl DebugSecret for ZeroizableUrl {}
impl Deref for ZeroizableUrl {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryInto<TargetEndpoint> for TargetEndpointBase {
    type Error = InitTargetEndpointError;

    fn try_into(self) -> Result<TargetEndpoint, Self::Error> {
        let scheme = self.url.expose_secret().scheme();
        match scheme {
            "http" | "https" => Ok(TargetEndpoint::Http(HttpTargetEndpoint(self))),
            "ws" | "wss" => Ok(TargetEndpoint::WebSocket(WebSocketTargetEndpoint(self))),
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
