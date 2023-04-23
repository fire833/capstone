//! Contains a wrapper struct for RustEmbed to bundle the static
//! UI assets with the binary, so that they can be served from the API

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "hub_router_ui/dist"]
pub struct WebUIAssets;
