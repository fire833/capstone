//! The API server which serves the UI and provides a configuration interface

use crate::hub::{Hub, HubState, HubMetadata};
use crate::logger::SEVERE_LOG_STORE;
use crate::routing::RoutingDecision;
use crate::schema::Session;
use crate::state::{HubRouterPrimitiveConfigs, HubRouterState};
use crate::ui::WebUIAssets;
use dashmap::DashMap;
use hyper::body::Bytes;
use hyper::{Client, Request, StatusCode, Uri};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::timeout;
use url::Url;
use utoipa::OpenApi;
use uuid::Uuid;
use warp::path::Tail;
use warp::reply::Response;
use warp::{reply, Filter};

/// Primary entrypoint for the API. Will run and provide information and capabilities to
/// update information on the running Hub programatically.
pub async fn hub_api_thread(
    state: Arc<HubRouterState>,
    sessions: Arc<DashMap<String, RoutingDecision>>,
) {
    let bind_tuple = match state.configs.read() {
        Ok(conf) => (conf.api_bind_ip, conf.api_bind_port),
        Err(err) => {
            warn!("RWLock was poisoned generating api bind tuple: {}", err);
            let conf = HubRouterPrimitiveConfigs::default();
            (conf.api_bind_ip, conf.api_bind_port)
        }
    };
    info!("starting api thread");
    let state_filter = warp::any().map(move || state.clone());
    let sessions_filter = warp::any().map(move || sessions.clone());

    let openapi_spec = warp::get()
        .and(warp::path!("swagger.json"))
        .and(warp::path::end())
        .and_then(serve_openapi);

    let get_hubs = warp::get()
        .and(warp::path!("api" / "hubs"))
        .and(warp::path::end())
        .and(state_filter.clone())
        .and_then(get_hubs);

    let create_hub = warp::post()
        .and(warp::path!("api" / "hubs"))
        .and(warp::path::end())
        .and(state_filter.clone())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and_then(create_hub);

    let delete_hub = warp::delete()
        .and(warp::path!("api" / "hubs" / Uuid))
        .and(warp::path::end())
        .and(state_filter.clone())
        .and_then(delete_hub);

    let get_config_values = warp::get()
        .and(warp::path!("api" / "config" / String))
        .and(warp::path::end())
        .and(state_filter.clone())
        .and_then(get_config);

    let set_config_values = warp::post()
        .and(warp::path!("api" / "config" / String / u64))
        .and(warp::path::end())
        .and(state_filter.clone())
        .and_then(set_config);

    let get_router_config = warp::get()
        .and(warp::path!("api" / "config"))
        .and(warp::path::end())
        .and(state_filter.clone())
        .and_then(get_entire_config);

    let set_router_config = warp::post()
        .and(warp::path!("api" / "config"))
        .and(warp::path::end())
        .and(warp::body::json::<HubRouterPrimitiveConfigs>())
        .and(state_filter.clone())
        .and_then(set_entire_config);

    let get_severe_logs = warp::get()
        .and(warp::path!("api" / "logs"))
        .and(warp::path::end())
        .and_then(get_logs);

    let get_sessions = warp::get()
        .and(warp::path!("api" / "sessions"))
        .and(warp::path::end())
        .and(sessions_filter.clone())
        .and_then(get_sessions);

    let aggregate_graphql_responses = warp::post()
        .and(warp::path!("api" / "graphql"))
        .and(warp::path::end())
        .and(warp::body::bytes())
        .and(state_filter.clone())
        .and_then(aggregate_graphql_responses);

    let aggregate_status_responses = warp::get()
        .and(warp::path!("api" / "hubs" / "status"))
        .and(warp::path::end())
        .and(state_filter.clone())
        .and_then(aggregate_status_responses);

    let get_ui = warp::get()
        .and(warp::path("ui"))
        .and(warp::path::tail())
        .and_then(serve_ui);

    let routes = get_hubs
        .or(create_hub)
        .or(delete_hub)
        .or(get_sessions)
        .or(get_ui)
        .or(aggregate_graphql_responses)
        .or(aggregate_status_responses)
        .or(set_config_values)
        .or(get_config_values)
        .or(get_router_config)
        .or(set_router_config)
        .or(get_severe_logs)
        .or(openapi_spec)
        .or(warp::any().map(|| {
            Ok(warp::reply::with_status(
                reply::reply(),
                StatusCode::NOT_FOUND,
            ))
        }))
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(vec!["GET", "POST", "OPTIONS"]),
        );

    warp::serve(routes).run(bind_tuple).await;
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_hubs,
        create_hub,
        delete_hub,
        get_sessions,
        set_config,
        get_config,
        get_entire_config,
        set_entire_config,
        get_logs,
    ),
    components(schemas(Hub, HubRouterState, HubState, HubMetadata)),
    info(
        description = "OpenAPI specification for the Hub Router API.",
        title = "Hub Router API",
        version = "0.0.1"
    )
)]
struct HubRouterAPIDocs;

async fn serve_openapi() -> Result<impl warp::Reply, warp::Rejection> {
    match HubRouterAPIDocs::openapi().to_pretty_json() {
        Ok(data) => Ok(warp::reply::with_status(data, StatusCode::OK)),
        Err(e) => Ok(warp::reply::with_status(
            e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

#[utoipa::path(get, path = "/api/hubs")]
async fn get_hubs(state: Arc<HubRouterState>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut lhubs: Vec<Hub> = vec![];

    for hub in state.hubs.iter() {
        let c = hub.clone();
        lhubs.push(c);
    }

    Ok(warp::reply::json(&lhubs))
}

#[derive(Serialize, Deserialize)]
struct HubNameAndURL {
    name: String,
    url: String,
}

#[utoipa::path(
    post,
    path = "/api/hubs",
    responses(
        (status = 200, description = "Registered Hub successfully"),
        (status = INTERNAL_SERVER_ERROR, description = "Unable to persist Hub"),
        (status = NOT_ACCEPTABLE, description = "Hub already registered"),
        (status = BAD_REQUEST, description = "Hub URL invalid")
    ),
    params(
        ("hub" = Hub, description = "Hub to insert"),
    )
)]
async fn create_hub(
    state: Arc<HubRouterState>,
    meta: HubNameAndURL,
) -> Result<impl warp::Reply, warp::Rejection> {
    let url = match Url::from_str(&meta.url) {
        Ok(url) => url,
        Err(err) => {
            return Ok(warp::reply::with_status(
                format!("Invalid hub URL: {} | {}", &meta.url, err),
                StatusCode::BAD_REQUEST,
            ));
        }
    };

    if state.hubs.iter().any(|e| &e.meta.url == &url) {
        return Ok(warp::reply::with_status(
            format!("hub at {} already registered", url.as_str()),
            StatusCode::NOT_ACCEPTABLE,
        ));
    } else {
        let new_hub = Hub::new_with_name(&meta.name, url);
        state.hubs.insert(new_hub.meta.uuid, new_hub);
        if let Err(e) = state.persist() {
            return Ok(warp::reply::with_status(
                format!("Unable to persist new hub: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }

    return Ok(warp::reply::with_status(
        format!("registered hub successfully"),
        StatusCode::CREATED,
    ));
}

#[utoipa::path(
    delete, 
    path = "/api/hubs/{uuid}", 
    responses(
        (status = 200, description = "Deleted Hub successfully"),
        (status = INTERNAL_SERVER_ERROR, description = "Unable to persist removed Hub"),
    ),
    params(
        ("uuid" = Hub, Path, description = "UUID of Hub to remove."),
    )
)]
async fn delete_hub(
    uuid: Uuid,
    state: Arc<HubRouterState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    state.hubs.remove(&uuid);
    if let Err(e) = state.persist() {
        return Ok(warp::reply::with_status(
            format!("Unable to persist removed hub: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    return Ok(warp::reply::with_status(
        format!("deleted hub successfully"),
        StatusCode::CREATED,
    ));
}

#[utoipa::path(
    get, 
    path = "/api/sessions",
    responses(
        (status = 200, description = "Returned current active sessions"),
    ),
    params()
)]
async fn get_sessions(
    sessions: Arc<DashMap<String, RoutingDecision>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut sess: Vec<Session> = vec![];

    for s in sessions.iter() {
        sess.push(Session::new(s.key(), &s.value().hub_endpoint));
    }

    Ok(warp::reply::json(&sess))
}

async fn serve_ui(tail: Tail) -> Result<impl warp::Reply, warp::Rejection> {
    let path = if tail.as_str() == "" {
        "index.html"
    } else {
        tail.as_str()
    };

    let maybe_file = WebUIAssets::get(path);
    match maybe_file {
        Some(file) => Ok(warp::reply::with_header(
            Response::new(file.data.into()),
            "Content-Type",
            mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string(),
        )),
        None => Err(warp::reject::not_found()),
    }
}

#[utoipa::path(post, 
    path = "/api/config/{key}/{value}",
    responses(
        (status = 200, description = "Set config value."),
    ),
    params(
        ("key" = String, Path, description = "Key of config value to be set."),
        ("value" = u64, Path, description = "Value of config value to be set."),
    )
)]
async fn set_config(
    key: String,
    value: u64,
    state: Arc<HubRouterState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if matches!(
        key.as_str(),
        "healthcheck_interval" | "reaper_interval" | "reaper_max_duration" | "healthcheck_timeout"
    ) {
        let res = if let Ok(mut conf) = state.configs.write() {
            match key.as_str() {
                "healthcheck_interval" => {
                    conf.healthcheck_thread_interval = value;
                    Ok(warp::reply::with_status(
                        "successfully set healthcheck thread interval".to_string(),
                        StatusCode::OK,
                    ))
                }
                "healthcheck_timeout" => {
                    conf.healthcheck_timeout = value;
                    Ok(warp::reply::with_status(
                        "successfully set healthcheck timeout".to_string(),
                        StatusCode::OK,
                    ))
                }
                "reaper_interval" => {
                    conf.reaper_thread_interval = value;
                    Ok(warp::reply::with_status(
                        "successfully set reaper thread interval".into(),
                        StatusCode::OK,
                    ))
                }
                "reaper_max_duration" => {
                    conf.reaper_thread_duration_max = value;
                    Ok(warp::reply::with_status(
                        "successfully set reaper max session duration".into(),
                        StatusCode::OK,
                    ))
                }
                _ => Ok(warp::reply::with_status(
                    "invalid config parameter to set".into(),
                    StatusCode::NOT_ACCEPTABLE,
                )),
            }
        } else {
            Ok(warp::reply::with_status(
                "unable to acquire write lock for configs".into(),
                StatusCode::NOT_ACCEPTABLE,
            ))
        };

        if let Err(e) = state.persist() {
            return Ok(warp::reply::with_status(
                format!("Unable to persist new hub: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        } else {
            res
        }
    } else {
        return Ok(warp::reply::with_status(
            "invalid config parameter to set".into(),
            StatusCode::NOT_ACCEPTABLE,
        ));
    }
}

#[utoipa::path(get, 
    path = "/api/config/{key}",
    responses(
        (status = 200, description = "Retrieved config value."),
    ),
    params(
        ("key" = String, Path, description = "Key of config value to be retrieved."),
    )
)]
async fn get_config(
    key: String,
    state: Arc<HubRouterState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if matches!(
        key.as_str(),
        "healthcheck_interval" | "reaper_interval" | "reaper_max_duration" | "healthcheck_timeout"
    ) {
        let res = if let Ok(conf) = state.configs.read() {
            match key.as_str() {
                "healthcheck_interval" => Ok(warp::reply::with_status(
                    conf.healthcheck_thread_interval.to_string(),
                    StatusCode::OK,
                )),
                "healthcheck_timeout" => Ok(warp::reply::with_status(
                    conf.healthcheck_timeout.to_string(),
                    StatusCode::OK,
                )),
                "reaper_interval" => Ok(warp::reply::with_status(
                    conf.reaper_thread_interval.to_string(),
                    StatusCode::OK,
                )),
                "reaper_max_duration" => Ok(warp::reply::with_status(
                    conf.reaper_thread_duration_max.to_string(),
                    StatusCode::OK,
                )),
                _ => Ok(warp::reply::with_status(
                    "invalid config parameter to set".into(),
                    StatusCode::NOT_ACCEPTABLE,
                )),
            }
        } else {
            Ok(warp::reply::with_status(
                "unable to acquire write lock for configs".into(),
                StatusCode::NOT_ACCEPTABLE,
            ))
        };

        if let Err(e) = state.persist() {
            return Ok(warp::reply::with_status(
                format!("Unable to persist new hub: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        } else {
            res
        }
    } else {
        return Ok(warp::reply::with_status(
            "invalid config parameter to set".into(),
            StatusCode::NOT_ACCEPTABLE,
        ));
    }
}

#[utoipa::path(get, 
    path = "/api/config",
    responses(
        (status = 200, description = "Retrieved config."),
    ),
    params()
)]
async fn get_entire_config(
    state: Arc<HubRouterState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match state.configs.read() {
        Ok(conf) => Ok(warp::reply::with_status(
            warp::reply::json(&*conf),
            StatusCode::OK,
        )),
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&format!("unable to acquire read lock for configs: {}", e)),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

#[utoipa::path(post, 
    path = "/api/config",
    responses(
        (status = 200, description = "Updated HubRouterState config object"),
    ),
    params(
        ("config" = HubRouterState, description = "A copy of the HubRouterState object that you wish to persist to disk."),
    )
)]
async fn set_entire_config(
    config: HubRouterPrimitiveConfigs,
    state: Arc<HubRouterState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let res = match state.configs.write() {
        Ok(mut conf) => {
            *conf = config;
            Ok(warp::reply::with_status("ok".into(), StatusCode::OK))
        }
        Err(e) => Ok(warp::reply::with_status(
            format!("unable to acquire read lock for configs: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    };

    if let Err(e) = state.persist() {
        return Ok(warp::reply::with_status(
            format!("Unable to persist configuration changes: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    res
}

#[utoipa::path(get, 
    path = "/api/logs",
    responses(
        (status = 200, description = "Returned all current, deduped warn/error logs for hub_router"),
    ),
)]
async fn get_logs() -> Result<impl warp::Reply, warp::Rejection> {
    match SEVERE_LOG_STORE.read() {
        Ok(store) => {
            let serialized = serde_json::to_string_pretty(&*store);
            match serialized {
                Ok(string) => {
                    return Ok(warp::reply::with_status(string, StatusCode::OK));
                }
                Err(e) => {
                    return Ok(warp::reply::with_status(
                        format!("Error serializing logs: {}", e),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                }
            }
        }
        Err(e) => {
            return Ok(warp::reply::with_status(
                format!("Unable to acquire read lock for logs: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AggregatedResponse {
    response: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AggregatedError {
    error: String,
}

impl<T> From<T> for AggregatedError
where
    T: ToString,
{
    fn from(value: T) -> Self {
        return AggregatedError {
            error: value.to_string(),
        };
    }
}

async fn make_single_aggregate_request(
    req: Request<Bytes>,
) -> Result<AggregatedResponse, AggregatedError> {
    let (parts, body_bytes) = req.into_parts();
    let uri_path = parts.uri.clone();
    let built_request = hyper::Request::from_parts(parts, hyper::Body::from(body_bytes));
    let client = Client::new();
    let response_future = client.request(built_request);
    let response = match timeout(Duration::from_secs(2), response_future).await {
        Ok(body) => body?,
        Err(_) => {
            return Err(format!("Request to {} timed out", uri_path.path()).into());
        }
    };

    let body = hyper::body::to_bytes(response.into_body()).await?;
    let as_string = String::from_utf8(body.to_vec())?;
    Ok(AggregatedResponse {
        response: as_string,
    })
}

async fn aggregate_request(
    reqs: Vec<Request<Bytes>>,
) -> Vec<Result<AggregatedResponse, AggregatedError>> {
    let mut join_set: JoinSet<Result<AggregatedResponse, AggregatedError>> = JoinSet::new();

    for request in reqs {
        join_set.spawn(async move { make_single_aggregate_request(request).await });
    }

    let mut serialized_json_responses: Vec<Result<AggregatedResponse, AggregatedError>> = vec![];

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(res) => {
                serialized_json_responses.push(res);
            }
            Err(err) => {
                serialized_json_responses.push(Err(err.to_string().into()));
            }
        }
    }

    return serialized_json_responses;
}

#[derive(Debug, Serialize, Deserialize)]
struct APIHubsStatusResponse {
    hub_status_response: Result<AggregatedResponse, AggregatedError>,
    router_hub_state: Hub,
}

async fn aggregate_status_responses(
    state: Arc<HubRouterState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let unvalidated_requests: Vec<_> = state
        .hubs
        .iter()
        .map(|h| {
            let uri = Uri::from_str(
                {
                    let mut status_endpoint = h.meta.url.clone();
                    status_endpoint.set_path("/status");
                    status_endpoint
                }
                .as_str(),
            )?;
            return Request::builder()
                .method("GET")
                .uri(uri)
                .body(hyper::body::Bytes::default());
        })
        .collect();

    let mut validated_requests: Vec<Request<Bytes>> = vec![];
    for req in unvalidated_requests {
        match req {
            Ok(request) => validated_requests.push(request),
            Err(err) => {
                return Ok(warp::reply::with_status(
                    warp::reply::html(format!("Error building request: {}", err)),
                    StatusCode::BAD_REQUEST,
                ));
            }
        }
    }

    let mut req_join_set: JoinSet<APIHubsStatusResponse> = JoinSet::new();

    for (req, hub) in validated_requests.into_iter().zip(state.hubs.iter()) {
        let cloned_hub = hub.clone();
        req_join_set.spawn(async move {
            let response = make_single_aggregate_request(req).await;
            APIHubsStatusResponse {
                hub_status_response: response,
                router_hub_state: cloned_hub,
            }
        });
    }

    let mut status_responses: Vec<APIHubsStatusResponse> = vec![];

    while let Some(result) = req_join_set.join_next().await {
        match result {
            Ok(res) => {
                status_responses.push(res);
            }
            Err(err) => {
                return Ok(warp::reply::with_status(
                    warp::reply::html(format!("Join error: {}", err)),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
        }
    }

    let joined = serde_json::to_string(&status_responses);

    match joined {
        Ok(response) => Ok(warp::reply::with_status(
            warp::reply::html(response),
            StatusCode::OK,
        )),
        Err(err) => Ok(warp::reply::with_status(
            warp::reply::html(format!("Error serializing aggregated responses: {}", err)),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

async fn aggregate_graphql_responses(
    graphql_request: Bytes,
    state: Arc<HubRouterState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let unvalidated_requests: Vec<_> = state
        .hubs
        .iter()
        .map(|h| {
            let uri = Uri::from_str(
                {
                    let mut status_endpoint = h.meta.url.clone();
                    status_endpoint.set_path("/graphql");
                    status_endpoint
                }
                .as_str(),
            )?;
            return Request::builder()
                .method("POST")
                .header("Content-Type", "application/json")
                .uri(uri)
                .body(graphql_request.clone());
        })
        .collect();

    let mut validated_requests: Vec<Request<Bytes>> = vec![];
    for req in unvalidated_requests {
        match req {
            Ok(request) => validated_requests.push(request),
            Err(err) => {
                return Ok(warp::reply::with_status(
                    warp::reply::html(format!("Error building request: {}", err)),
                    StatusCode::BAD_REQUEST,
                ));
            }
        }
    }

    let responses = aggregate_request(validated_requests).await;

    let joined = serde_json::to_string(&responses);

    match joined {
        Ok(response) => Ok(warp::reply::with_status(
            warp::reply::html(response),
            StatusCode::OK,
        )),
        Err(err) => Ok(warp::reply::with_status(
            warp::reply::html(format!("Error serializing aggregated responses: {}", err)),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}
