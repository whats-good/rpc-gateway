#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

pub mod chain_handler;
pub mod cli;
pub mod cors;
pub mod gateway;
pub mod lazy_request;
pub mod load_balancer;
pub mod logging;
pub mod metrics;
pub mod request_pool;
pub mod server;
pub mod upstream;
