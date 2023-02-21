use std::fmt::Debug;

#[derive(Debug)]
pub enum RoutingError {
    NoHealthyNodes(String),
    UnableToSatisfyCapabilities(String),
    MalformedRequestPath(String),
    NoCapacity(String),
}

#[derive(Debug)]
pub enum HubRouterError {
    RoutingError(RoutingError),
    HyperError(hyper::Error),
    DeserializationError(serde_json::Error),
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
