#![warn(rust_2018_compatibility, future_incompatible, nonstandard_style)]

extern crate tile_geom as geom;

use clap::Parser;

#[macro_use]
mod text;

mod action;
mod assets;
mod defs;
mod draw;
mod planet;
mod screen;
mod sim;
mod ui;

use bevy::{prelude::*, window::PresentMode, winit::WinitSettings};

#[derive(Clone, Parser, Debug)]
#[clap(author, version)]
struct Args {
    /// Open map editing tools
    #[clap(long)]
    edit_map: bool,
}

fn main() {
    let args = Args::parse();

    App::new()
        .insert_resource(DefaultTaskPoolOptions::with_num_threads(2))
        .add_plugins(DefaultPlugins)
        .add_plugin(text::TextPlugin)
        .add_plugin(assets::AssetsPlugin)
        .add_plugin(screen::ScreenPlugin)
        .add_plugin(ui::UiPlugin {
            edit_map: args.edit_map,
        })
        .add_plugin(InspectorPlugin)
        .add_plugin(draw::DrawPlugin)
        .add_plugin(action::ActionPlugin)
        .add_plugin(sim::SimPlugin)
        .insert_resource(WinitSettings::game())
        .insert_resource(WindowDescriptor {
            present_mode: PresentMode::Mailbox,
            ..Default::default()
        })
        .run();
}

#[derive(Clone, Copy, Debug)]
pub struct InspectorPlugin;

#[cfg(feature = "inspector")]
impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());
    }
}

#[cfg(not(feature = "inspector"))]
impl Plugin for InspectorPlugin {
    fn build(&self, _app: &mut App) {}
}
