use super::settings::Settings;
use crate::{
    chain::{Chain, ChainId},
    target_endpoint::{InitTargetEndpointError, TargetEndpoint, TargetEndpointBase},
};
use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::{collections::HashMap, time::Duration};
use thiserror::Error;
use url::{ParseError, Url};

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct RawChain {
    block_time: Option<u64>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct RawTargetEndpoint {
    chain_id: u64,
    url: String, // TODO: should this be URL instead?
}

impl RawTargetEndpoint {
    pub fn try_into_target_endpoint(
        self,
        name: String,
        chains_map: &mut HashMap<ChainId, &'static Chain>,
    ) -> Result<TargetEndpoint, SettingsError> {
        let chain_id: ChainId = self.chain_id.into();
        let new_chain: Chain = chain_id.into();
        let new_chain: &'static Chain = new_chain.leak();
        let entry = chains_map.entry(chain_id).or_insert(new_chain);
        let chain: &'static Chain = *entry;
        let url = Url::parse(&self.url)?;
        let target_endpoint_base = TargetEndpointBase { name, chain, url };
        let target_endpoint: TargetEndpoint = target_endpoint_base.try_into()?;
        Ok(target_endpoint)
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct RawSettings {
    chains: Option<HashMap<u64, RawChain>>,
    target_endpoints: HashMap<String, RawTargetEndpoint>,
}

impl RawSettings {
    pub fn from_config_file(path: &str) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(path))
            .build()?;

        s.try_deserialize::<RawSettings>()
    }
}

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("non existent chain id")]
    NonExistentChain(ChainId),

    #[error("url parse error")]
    UrlParseError(#[from] ParseError),

    #[error("init target endpoint error")]
    InitTargetEndpointError(#[from] InitTargetEndpointError),
}

impl TryFrom<RawSettings> for Settings {
    type Error = SettingsError;

    fn try_from(value: RawSettings) -> Result<Self, Self::Error> {
        let mut chains_map: HashMap<ChainId, &'static Chain> = value
            .chains
            .unwrap_or_default()
            .into_iter()
            .map(|(raw_chain_id, raw_chain)| {
                // TODO: log overwrites
                let chain = Chain {
                    chain_id: raw_chain_id.into(),
                    block_time: raw_chain.block_time.map(Duration::from_secs),
                };
                (chain.chain_id, chain.leak())
            })
            .collect();

        // // TODO: this might not be the best way to collect all validation issues
        let target_endpoints: Vec<TargetEndpoint> = value
            .target_endpoints
            .into_iter()
            .map(|(name, raw_endpoint)| {
                raw_endpoint.try_into_target_endpoint(name, &mut chains_map)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Settings::new(target_endpoints.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn read_raw_settings_chains_from_toml() {
        let raw_settings = RawSettings::from_config_file("example_configs/example").unwrap();
        let actual = raw_settings.chains.unwrap();
        let expected = [
            (
                1,
                RawChain {
                    block_time: Some(12),
                },
            ),
            (
                8453,
                RawChain {
                    block_time: Some(2),
                },
            ),
            (
                11155111,
                RawChain {
                    block_time: Some(12),
                },
            ),
            (
                84532,
                RawChain {
                    block_time: Some(2),
                },
            ),
        ];
        assert_eq!(actual, expected.into(),);
    }

    #[test]
    pub fn read_raw_settings_target_endpoints_from_toml() {
        let actual = RawSettings::from_config_file("example_configs/example")
            .unwrap()
            .target_endpoints;
        let expected = [
            (
                "first_endpoint".to_string(),
                RawTargetEndpoint {
                    url: "http://localhost:8080".to_string(),
                    chain_id: 1,
                },
            ),
            (
                "second_endpoint".to_string(),
                RawTargetEndpoint {
                    url: "http://localhost:8081".to_string(),
                    chain_id: 1,
                },
            ),
            (
                "third_endpoint".to_string(),
                RawTargetEndpoint {
                    url: "http://localhost:8082".to_string(),
                    chain_id: 8453,
                },
            ),
            (
                "fourth_endpoint".to_string(),
                RawTargetEndpoint {
                    url: "http://localhost:8083".to_string(),
                    chain_id: 84532,
                },
            ),
            (
                "fifth_endpoint".to_string(),
                RawTargetEndpoint {
                    url: "ws://localhost:8084".to_string(),
                    chain_id: 84532,
                },
            ),
        ]
        .into();
        assert_eq!(actual, expected);
    }

    #[test]
    pub fn raw_settings_to_settings() {
        // TODO: top level chains array list should enforce unique chain ids in the raw file
        // TODO: top level endpoints array list should enforce unique names in the raw file
        let raw_settings = RawSettings::from_config_file("example_configs/example").unwrap();
        let actual_settings: Settings = raw_settings.try_into().unwrap();
        let eth_mainnet = Chain::new(1.into(), Duration::from_secs(12).into()).leak();

        let first_endpoint: TargetEndpoint = TargetEndpointBase {
            name: "first_endpoint".to_string(),
            url: Url::parse("http://localhost:8080".into()).unwrap(),
            chain: eth_mainnet,
        }
        .try_into()
        .unwrap();

        let second_endpoint: TargetEndpoint = TargetEndpointBase {
            name: "second_endpoint".to_string(),
            url: Url::parse("http://localhost:8081".into()).unwrap(),
            chain: eth_mainnet,
        }
        .try_into()
        .unwrap();

        let base_mainnet = Chain::new(8453.into(), Duration::from_secs(2).into()).leak();

        let third_endpoint: TargetEndpoint = TargetEndpointBase {
            name: "third_endpoint".to_string(),
            url: Url::parse("http://localhost:8082".into()).unwrap(),
            chain: base_mainnet,
        }
        .try_into()
        .unwrap();

        let base_sepolia = Chain::new(84532.into(), Duration::from_secs(2).into()).leak();

        let fourth_endpoint: TargetEndpoint = TargetEndpointBase {
            name: "fourth_endpoint".to_string(),
            url: Url::parse("http://localhost:8083".into()).unwrap(),
            chain: base_sepolia,
        }
        .try_into()
        .unwrap();

        let fifth_endpoint: TargetEndpoint = TargetEndpointBase {
            name: "fifth_endpoint".to_string(),
            url: Url::parse("ws://localhost:8084".into()).unwrap(),
            chain: base_sepolia,
        }
        .try_into()
        .unwrap();

        let chains_to_targets = vec![
            first_endpoint.try_into().unwrap(),
            second_endpoint.try_into().unwrap(),
            third_endpoint.try_into().unwrap(),
            fourth_endpoint.try_into().unwrap(),
            fifth_endpoint,
        ]
        .into();

        // TODO: test errors
        // let eth_sepolia = Chain::new(11155111.into(), Duration::from_secs(12).into()).leak();
        // let expected_eth_sepolia_targets = TargetEndpointsForChain {
        //     chain: eth_sepolia,
        //     http: vec![],
        //     web_socket: vec![],
        // };
        // TODO: should we include chains that have no providers in the settings? probably yes
        // TODO: remove the chain_id_not_found requirement
        // TODO: add tests for configs with optional fields

        // let chains_to_targets = ChainsToEndpoints::new(HashMap::from([
        //     (base_mainnet, expected_base_mainnet_targets),
        //     (eth_mainnet, expected_eth_mainnet_targets),
        //     (base_sepolia, expected_base_sepolia_targets),
        // ]));

        let expected_settings = Settings::new(chains_to_targets);

        assert_eq!(actual_settings, expected_settings);
    }
}
