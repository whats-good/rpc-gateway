use crate::{
    chain::Chain,
    target_endpoint::{HttpTargetEndpoint, TargetEndpoint, WebSocketTargetEndpoint},
};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

#[derive(Debug, PartialEq, Eq)]
pub struct TargetEndpointsForChain {
    pub chain: &'static Chain,
    pub http: Vec<HttpTargetEndpoint>,
    pub web_socket: Vec<WebSocketTargetEndpoint>,
}

impl TargetEndpointsForChain {
    pub fn new(chain: &'static Chain) -> Self {
        TargetEndpointsForChain {
            chain,
            http: vec![],
            web_socket: vec![],
        }
    }

    pub fn add(&mut self, endpoint: TargetEndpoint) {
        match endpoint {
            TargetEndpoint::Http(endpoint) => self.http.push(endpoint),
            TargetEndpoint::WebSocket(endpoint) => self.web_socket.push(endpoint),
        };
    }
}

type ChainsToEndpointsInner = HashMap<&'static Chain, TargetEndpointsForChain>;

#[derive(Debug, PartialEq, Eq)]
pub struct ChainsToEndpoints(ChainsToEndpointsInner);

impl ChainsToEndpoints {
    pub fn new(inner: ChainsToEndpointsInner) -> Self {
        Self(inner)
    }
}

impl Deref for ChainsToEndpoints {
    type Target = ChainsToEndpointsInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChainsToEndpoints {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<TargetEndpoint>> for ChainsToEndpoints {
    fn from(value: Vec<TargetEndpoint>) -> Self {
        let mut chains_to_endpoints_inner: ChainsToEndpointsInner = HashMap::new();

        value.into_iter().for_each(|endpoint| {
            let chain: &'static Chain = endpoint.chain;

            match chains_to_endpoints_inner.get_mut(chain) {
                Some(v) => {
                    v.add(endpoint);
                }
                None => {
                    let mut target_endpoints = TargetEndpointsForChain::new(chain);
                    target_endpoints.add(endpoint);
                    chains_to_endpoints_inner.insert(chain, target_endpoints);
                }
            };
        });

        ChainsToEndpoints(chains_to_endpoints_inner)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Settings {
    pub chains_to_targets: ChainsToEndpoints,
}

impl Settings {
    pub fn leak(self) -> &'static Settings {
        Box::leak(Box::new(self))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::target_endpoint::TargetEndpointBase;
    use lazy_static::lazy_static;
    use url::Url;

    lazy_static! {
        static ref CHAIN_ONE: Chain = Chain::new(1.into(), None);
        static ref CHAIN_TWO: Chain = Chain::new(2.into(), None);
    }

    lazy_static! {
        static ref TEST_SETTINGS: Settings = get_settings();
    }

    fn get_settings() -> Settings {
        // TODO: test errors
        // TODO: the names should be enforced unique, via a struct UniqueString
        // TODO: top level chains array list should enforce unique chain ids
        // TODO: top level endpoints array list should enforce unique names
        let base_endpoints = vec![
            TargetEndpointBase {
                chain: &CHAIN_ONE,
                name: "My First Endpoint".to_string(),
                url: Url::parse("http://something1.com").unwrap(),
            },
            TargetEndpointBase {
                chain: &CHAIN_ONE,
                name: "My Second Endpoint".to_string(),
                url: Url::parse("https://something2.com").unwrap(),
            },
            TargetEndpointBase {
                chain: &CHAIN_TWO,
                name: "My Third Endpoint".to_string(),
                url: Url::parse("https://something3.com").unwrap(),
            },
            TargetEndpointBase {
                chain: &CHAIN_TWO,
                name: "My Fourth Endpoint".to_string(),
                url: Url::parse("http://something4.com").unwrap(),
            },
            TargetEndpointBase {
                chain: &CHAIN_TWO,
                name: "My Fifth Endpoint".to_string(),
                url: Url::parse("ws://something5.com").unwrap(),
            },
            TargetEndpointBase {
                chain: &CHAIN_TWO,
                name: "My Sixth Endpoint".to_string(),
                url: Url::parse("wss://something6.com").unwrap(),
            },
        ];

        let target_endpoints: Vec<TargetEndpoint> = base_endpoints
            .into_iter()
            .map(|x| x.try_into().unwrap())
            .collect();

        Settings {
            chains_to_targets: target_endpoints.into(),
        }
    }

    #[test]
    fn should_correctly_categorize_http_and_https() {
        let settings: &'static Settings = &TEST_SETTINGS;
        let first_chain_targets = settings
            .chains_to_targets
            .get(&CHAIN_ONE as &Chain)
            .unwrap();

        assert_eq!(
            first_chain_targets.http.len(),
            2,
            "first chain target http length does not match"
        );

        let second_chain_targets = settings
            .chains_to_targets
            .get(&CHAIN_TWO as &Chain)
            .unwrap();

        assert_eq!(
            second_chain_targets.http.len(),
            2,
            "second chain target http length does not match"
        );
    }

    #[test]
    fn should_correctly_categorize_ws_and_wss() {
        let settings: &'static Settings = &TEST_SETTINGS;
        let first_chain_targets = settings
            .chains_to_targets
            .get(&CHAIN_ONE as &Chain)
            .unwrap();

        assert_eq!(
            first_chain_targets.web_socket.len(),
            0,
            "first chain target websocket length does not match"
        );

        let second_chain_targets = settings
            .chains_to_targets
            .get(&CHAIN_TWO as &Chain)
            .unwrap();

        assert_eq!(
            second_chain_targets.web_socket.len(),
            2,
            "second chain target websocket length does not match"
        );
    }
}