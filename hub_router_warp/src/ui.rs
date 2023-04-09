use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "hubrouter_ui/build"]
pub struct WebUIAssets;