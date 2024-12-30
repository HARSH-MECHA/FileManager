use std::collections::HashMap;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

use mctk_core::reexports::cosmic_text;
use mctk_core::AssetParams;
use mctk_smithay::layer_shell::layer_surface::LayerOptions;
use mctk_smithay::layer_shell::layer_window;
use mctk_smithay::{WindowInfo, WindowOptions};
use smithay_client_toolkit::shell::wlr_layer;
mod gui;
use gui::{FileManager, FileManagerParams};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("debug"));
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(env_filter)
        .init();

    let mut fonts = cosmic_text::fontdb::Database::new();
    fonts.load_system_fonts();

    let mut assets: HashMap<String, AssetParams> = HashMap::new();
    let svgs: HashMap<String, String> = HashMap::new();

    let namespace = "mctk.file_manager".to_string();

    let layer_shell_opts = LayerOptions {
        anchor: wlr_layer::Anchor::TOP | wlr_layer::Anchor::LEFT,
        layer: wlr_layer::Layer::Overlay,
        keyboard_interactivity: wlr_layer::KeyboardInteractivity::Exclusive,
        namespace: Some(namespace.clone()),
        zone: 0,
    };

    let window_info = WindowInfo {
        id: "mctk.file_manager".to_string(),
        title: "File Manager".to_string(),
        namespace,
    };

    let window_opts = WindowOptions {
        height: 480,
        width: 480,
        scale_factor: 1.0,
    };
    
    assets.insert(
        "fold_icon".to_string(),
        AssetParams::new("src/assets/icons/fold.png".to_string()),
    );

    assets.insert(
        "file_icon".to_string(),
        AssetParams::new("src/assets/icons/file.png".to_string()),
    );

    assets.insert(
        "arrow_icon".to_string(),
        AssetParams::new("src/assets/icons/arrow.png".to_string()),
    );

    assets.insert(
        "back_icon".to_string(),
        AssetParams::new("src/assets/icons/Back.png".to_string()),
    );

    assets.insert(
        "add_icon".to_string(),
        AssetParams::new("src/assets/icons/add_icon.png".to_string()),
    );

    assets.insert(
        "dots_icon".to_string(),
        AssetParams::new("src/assets/icons/dots.png".to_string()),
    );

    assets.insert(
        "pdf_icon".to_string(),
        AssetParams::new("src/assets/icons/pdf.png".to_string()),
    );

    assets.insert(
        "img_icon".to_string(),
        AssetParams::new("src/assets/icons/image.png".to_string()),
    );

    let (mut app, mut event_loop, ..) =
        layer_window::LayerWindow::open_blocking::<FileManager, FileManagerParams>(
            layer_window::LayerWindowParams {
                window_info,
                window_opts,
                fonts,
                assets,
                svgs,
                layer_shell_opts,
                ..Default::default()
            },
            FileManagerParams {},
        );

    loop {
        event_loop
            .dispatch(Duration::from_millis(16), &mut app)
            .unwrap();
    }
}
