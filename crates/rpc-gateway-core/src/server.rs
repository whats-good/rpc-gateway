use crate::{
    cors::cors_middleware,
    gateway::{Gateway, GatewayRequest},
    lazy_request::PreservedRequest,
};
use actix_web::{App, HttpResponse, HttpServer, Result, web};
use metrics::{counter, histogram};
use rpc_gateway_config::{Config, ProjectConfig};
use rpc_gateway_rpc::{error::RpcError, response::Response};
use std::sync::{Arc, LazyLock};
use std::{collections::HashMap, time::Instant};
use tracing::{info, instrument, warn};

// TODO: use Result<HttpResponse> instead of unwrap everywhere.

static INVALID_REQUEST_BODY: LazyLock<String> = LazyLock::new(|| {
    simd_json::to_string(&Response::error(RpcError::internal_error_with(
        "Invalid JSON-RPC request",
    )))
    .unwrap()
    .into()
});

#[inline]
fn track_http_response(
    chain_id: u64, // TODO: consider using static strings here.
    gateway_project: &String,
    response_category: &'static str,
    start_time: Instant,
) {
    counter!("http_response_total",
        "chain_id" => chain_id.to_string(),
        "gateway_project" => gateway_project.clone(),
        "response_category" => response_category,
    )
    .increment(1);

    let duration = start_time.elapsed();

    histogram!("http_response_latency_seconds",
        "chain_id" => chain_id.to_string(),
        "gateway_project" => gateway_project.clone(),
        "response_category" => response_category,
    )
    .record(duration.as_secs_f64());
}

#[instrument(skip(gateway, start_time))]
async fn handle_rpc_request_inner(
    chain_id: u64,
    query: web::Query<HashMap<String, String>>,
    body: web::Bytes,
    gateway: web::Data<Arc<Gateway>>,
    project_config: ProjectConfig,
    start_time: Instant,
) -> HttpResponse {
    let project_key = query.get("key").cloned();
    let project_name = project_config.name.clone();
    let preserved_request = match PreservedRequest::try_from(body) {
        Ok(preserved_request) => preserved_request,
        Err(_) => {
            warn!("Failed to parse request body");

            track_http_response(chain_id, &project_name, "invalid_request", start_time);

            return HttpResponse::Ok().body(INVALID_REQUEST_BODY.clone());
        }
    };
    let gateway_request =
        GatewayRequest::new(project_config, project_key, chain_id, preserved_request);

    // TODO: when the gateway response is None, don't just respond with an error. Respond with 200 and an empty body instead.

    match gateway.handle_request(gateway_request).await {
        Some(response) => {
            let body = simd_json::to_string(&response).unwrap();

            // TODO: single_response can actually be an invalid_request response.
            // this could be coming directly from the upstream,
            // or literally from RpcCall::Invalid. figure out how to integrate them into the metrics.

            let response_category = match &response {
                Response::Single(_) => "rpc_call_single",
                Response::Batch(_) => "rpc_call_batch",
            };

            track_http_response(chain_id, &project_name, response_category, start_time);

            HttpResponse::Ok().body(body)
        }
        None => {
            track_http_response(chain_id, &project_name, "notification_ack", start_time);
            HttpResponse::Ok().body("")
        }
    }
}

async fn handle_rpc_request_with_project(
    path: web::Path<(String, u64)>,
    query: web::Query<HashMap<String, String>>,
    body: web::Bytes,
    gateway: web::Data<Arc<Gateway>>,
) -> HttpResponse {
    let start_time = Instant::now();
    let (project_name, chain_id) = path.into_inner();

    let project_config = match gateway.config.projects.get(&project_name) {
        Some(project_config) => project_config,
        None => {
            track_http_response(
                chain_id,
                &project_name,
                "proxy_project_not_found",
                start_time,
            );

            let body = simd_json::to_string(&Response::error(RpcError::internal_error_with(
                "Project not found",
            )))
            .unwrap();
            return HttpResponse::Ok().body(body);
        }
    }
    .clone();

    handle_rpc_request_inner(chain_id, query, body, gateway, project_config, start_time).await
}

async fn handle_rpc_request_without_project(
    path: web::Path<u64>,
    query: web::Query<HashMap<String, String>>,
    body: web::Bytes,
    gateway: web::Data<Arc<Gateway>>,
) -> HttpResponse {
    // TODO: what's the performance impact of these timers? Should we only optionally run them?
    let start_time = Instant::now();
    let chain_id = path.into_inner();
    let project_config = gateway.config.projects.get("default").unwrap().clone(); // TODO: make this a function on a ProjectsConfig struct.

    handle_rpc_request_inner(chain_id, query, body, gateway, project_config, start_time).await
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
