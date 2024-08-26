use std::collections::HashMap;

use derive_more::derive::Constructor;

use crate::{
    chain::Chain,
    target_endpoint::{HttpTargetEndpoint, TargetEndpoint, WebSocketTargetEndpoint},
};

#[derive(Debug)]
pub struct TargetEndpointsForChain {
    chain: &'static Chain,
    http: Vec<HttpTargetEndpoint>,
    web_socket: Vec<WebSocketTargetEndpoint>,
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

type ChainsToEndpoints = HashMap<&'static Chain, TargetEndpointsForChain>;

#[derive(Constructor, Debug)]
pub struct Config {
    pub target_endpoints: ChainsToEndpoints,
}

#[derive(Constructor)]
pub struct ConfigOptions {
    target_endpoints: Vec<TargetEndpoint>,
}

fn get_chains_to_endpoints(target_endpoints: Vec<TargetEndpoint>) -> ChainsToEndpoints {
    let mut chains_to_endpoints: ChainsToEndpoints = HashMap::new();

    target_endpoints.into_iter().for_each(|endpoint| {
        let chain: &'static Chain = endpoint.chain;

        match chains_to_endpoints.get_mut(chain) {
            Some(v) => {
                v.add(endpoint);
            }
            None => {
                let mut target_endpoints = TargetEndpointsForChain::new(chain);
                target_endpoints.add(endpoint);
                chains_to_endpoints.insert(chain, target_endpoints);
            }
        };
    });

    chains_to_endpoints
}

impl From<ConfigOptions> for Config {
    fn from(options: ConfigOptions) -> Self {
        Self {
            target_endpoints: get_chains_to_endpoints(options.target_endpoints),
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use secrecy::Secret;
    use url::Url;

    use crate::{
        chain::ChainId,
        target_endpoint::{self, TargetEndpointBase},
    };

    use super::*;

    #[test]
    fn test1() {
        let mut x = 0;
        let chains: [&'static Chain; 10] = [0; 10].map(|_| {
            x += 1;
            Chain::new(format!("Chain{x}"), Duration::from_secs(12), x.into()).leak()
        });

        // TODO: the names should be enforced unique, via a struct UniqueString

        let base_endpoints = vec![
            TargetEndpointBase {
                chain: chains[0],
                name: "My First Endpoint".to_string(),
                url: Secret::new(Url::parse("https://something1.com").unwrap().into()),
            },
            TargetEndpointBase {
                chain: chains[0],
                name: "My Second Endpoint".to_string(),
                url: Secret::new(Url::parse("https://something2.com").unwrap().into()),
            },
            TargetEndpointBase {
                chain: chains[0],
                name: "My Third Endpoint".to_string(),
                url: Secret::new(Url::parse("https://something3.com").unwrap().into()),
            },
            TargetEndpointBase {
                chain: chains[1],
                name: "My Fourth Endpoint".to_string(),
                url: Secret::new(Url::parse("https://something4.com").unwrap().into()),
            },
            TargetEndpointBase {
                chain: chains[1],
                name: "My Fifth Endpoint".to_string(),
                url: Secret::new(Url::parse("ws://something5.com").unwrap().into()),
            },
            TargetEndpointBase {
                chain: chains[1],
                name: "My Sixth Endpoint".to_string(),
                url: Secret::new(Url::parse("wss://something6.com").unwrap().into()),
            },
        ];

        let target_endpoints: Vec<TargetEndpoint> = base_endpoints
            .into_iter()
            .map(|x| x.try_into().unwrap())
            .collect();

        println!("{:?}", target_endpoints);

        let config_options = ConfigOptions::new(target_endpoints);
        let config: Config = config_options.into();

        println!("{config:#?}");
    }
}
