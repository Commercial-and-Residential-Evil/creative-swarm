//! Whirled Peas Visualiser: A Wordless Poem in Light and Sound
//!
//! Main entry point for the Bevy application.
//! This module configures the window, initializes default plugins,
//! and registers the main WhirledPeasPlugin.

use bevy::prelude::*;

#[cfg(not(target_os = "android"))]
use bevy::render::settings::{RenderCreation, WgpuSettings};
#[cfg(not(target_os = "android"))]
use bevy::render::RenderPlugin;

use whirled_peas::WhirledPeasPlugin;

/// Application entry point.
///
/// On desktop: Initializes the Bevy app with custom window configuration
/// and Vulkan rendering backend.
///
/// On Android: Uses the native window provided by GameActivity with
/// automatic rendering backend selection.
#[bevy_main]
fn main() {
    let mut app = App::new();

    #[cfg(target_os = "android")]
    {
        // Android: Use default plugins with minimal configuration
        // The native window is provided by GameActivity
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Whirled Peas".into(),
                        // Android handles resolution automatically
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        );
    }

    #[cfg(not(target_os = "android"))]
    {
        // Desktop: Custom window and render configuration
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Whirled Peas Visualiser: A Wordless Poem in Light and Sound".into(),
                        resolution: (1920.0, 1080.0).into(),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(bevy::render::settings::Backends::VULKAN),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        );
    }

    app.add_plugins(WhirledPeasPlugin).run();
}
