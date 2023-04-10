use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "hub_router_ui/dist"]
pub struct WebUIAssets;