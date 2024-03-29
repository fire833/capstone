///! An error wrapper for the Hub Router, so that errors from libraries which
///! we use can be `into`'d and homogenized to a single error type, which
///! we can back to a test for nicely formatted errors when things go wrong.
use std::fmt::Debug;


#[derive(Debug)]
pub enum RoutingError {
    NoHealthyNodes(String),
    UnableToSatisfyCapabilities(String),
    MalformedRequestPath(String),
    NoDecision(String),
}

#[derive(Debug)]
pub enum HubRouterError {
    RoutingError(RoutingError),
    HyperError(hyper::Error),
    DeserializationError(serde_json::Error),
    SessionCreationError(String),
    Internal(String),
}

impl HubRouterError {
    pub fn wrap_err<T, E>(result: Result<T, E>) -> Result<T, HubRouterError>
    where
        E: Debug + Into<HubRouterError>,
    {
        match result {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        }
    }
}

impl From<RoutingError> for HubRouterError {
    fn from(value: RoutingError) -> Self {
        HubRouterError::RoutingError(value)
    }
}

impl From<hyper::Error> for HubRouterError {
    fn from(value: hyper::Error) -> Self {
        HubRouterError::HyperError(value)
    }
}

impl From<serde_json::Error> for HubRouterError {
    fn from(value: serde_json::Error) -> Self {
        HubRouterError::DeserializationError(value)
    }
}
