use crate::{
    cors::cors_middleware,
    gateway::{Gateway, GatewayCall},
    lazy_request::{LazyRequest, PreservedCall},
};
use actix_web::{App, HttpResponse, HttpServer, Result, web};
use rpc_gateway_config::{Config, ProjectConfig};
use rpc_gateway_rpc::{error::RpcError, response::Response};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, instrument, warn};

#[instrument(skip(gateway))]
#[inline]
async fn handle_rpc_request_inner(
    chain_id: u64,
    query: web::Query<HashMap<String, String>>,
    body: web::Bytes,
    gateway: web::Data<Arc<Gateway>>,
    project_config: ProjectConfig,
) -> Result<String> {
    let project_key = query.get("key").cloned();

    let lazy_request = LazyRequest::try_from(body).map_err(|_| {
        warn!("Failed to parse request body");
        actix_web::error::ErrorBadRequest("Invalid JSON-RPC request")
    })?;

    let preserved_call = PreservedCall::try_from(lazy_request).map_err(|_| {
        warn!("Failed to parse request body");
        actix_web::error::ErrorBadRequest("Invalid JSON-RPC request")
    })?;

    let gateway_request = GatewayCall::new(project_config, project_key, chain_id, preserved_call);
    // TODO: the response doesn't need to be parsed beforehand. see if we can do more bytes shenanigans.

    let response = gateway
        .handle_request(gateway_request)
        .await
        .unwrap_or(Response::error(RpcError::internal_error_with(
            "Internal server error",
        )));

    let response_string = serde_json::to_string(&response)?;

    Ok(response_string)
}

async fn handle_rpc_request_with_project(
    path: web::Path<(String, u64)>,
    query: web::Query<HashMap<String, String>>,
    body: web::Bytes,
    gateway: web::Data<Arc<Gateway>>,
) -> Result<String> {
    let (project_name, chain_id) = path.into_inner();

    let project_config = match gateway.config.projects.get(&project_name) {
        Some(project_config) => project_config.clone(),
        None => {
            return Ok(serde_json::to_string(&Response::error(
                RpcError::internal_error_with("Project not found"),
            ))?);
        }
    };

    handle_rpc_request_inner(chain_id, query, body, gateway, project_config).await
}

async fn handle_rpc_request_without_project(
    path: web::Path<u64>,
    query: web::Query<HashMap<String, String>>,
    body: web::Bytes,
    gateway: web::Data<Arc<Gateway>>,
) -> Result<String> {
    let chain_id = path.into_inner();
    let project_config = gateway.config.projects.get("default").unwrap().clone(); // TODO: make this a function on a ProjectsConfig struct.

    handle_rpc_request_inner(chain_id, query, body, gateway, project_config).await
}

async fn liveness_probe() -> Result<String> {
    // TODO: implement real liveness probes.
    Ok("OK".to_string())
}

async fn readiness_probe() -> Result<String> {
    // TODO: implement readiness probes.
    Ok("OK".to_string())
}

pub struct GatewayServer {
    gateway: Arc<Gateway>,
    config: Arc<Config>,
}

impl GatewayServer {
    pub fn new(gateway: Arc<Gateway>, config: Arc<Config>) -> Self {
        Self { gateway, config }
    }

    pub async fn start(self) -> std::io::Result<()> {
        let config = self.config.clone();

        info!(
            server = ?config.server,
            metrics = ?config.metrics,
            chains = ?config.chains,
            cors = ?config.cors,
            projects = ?config.projects,
            upstream_health_checks = ?config.upstream_health_checks,
            error_handling = ?config.error_handling,
            canned_responses = ?config.canned_responses,
            cache = ?config.cache,
            load_balancing = ?config.load_balancing,
            request_coalescing = ?config.request_coalescing,
            logging = ?config.logging,
            "Starting server"
        );

        let host = self.config.server.host.clone();
        let port = self.config.server.port;
        HttpServer::new(move || {
            let cors = cors_middleware(&self.config.cors);
            let gateway = self.gateway.clone();

            App::new()
                .app_data(web::Data::new(gateway.clone()))
                .route("/health", web::get().to(liveness_probe))
                .route("/health/liveness", web::get().to(liveness_probe))
                .route("/health/readiness", web::get().to(readiness_probe))
                .route(
                    "/{project_name}/{chain_id}",
                    web::post().to(handle_rpc_request_with_project),
                )
                .route(
                    "/{chain_id}",
                    web::post().to(handle_rpc_request_without_project),
                )
                .default_service(
                    web::route().to(|| async { HttpResponse::NotFound().body("404 Not Found") }),
                )
                .wrap(cors)
        })
        .bind((host, port))?
        .run()
        .await
    }
}
