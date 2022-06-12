use bevy::{
    app::AppExit,
    input::{keyboard::KeyboardInput, ElementState},
    prelude::*,
};
use bevy_egui::{
    egui::{self, FontData, FontDefinitions, FontFamily},
    EguiContext, EguiPlugin, EguiSettings,
};
use std::collections::{HashMap, VecDeque};

use crate::{
    defs::Biome,
    screen::{CursorMode, HoverTile, OccupiedScreenSpace},
};
use crate::{defs::StructureKind, planet::Planet};
use crate::{msg::MsgKind, sim::ManagePlanet};

#[derive(Clone, Copy, Debug)]
pub struct UiPlugin {
    pub edit_map: bool,
}

#[derive(Clone, Default, Debug)]
pub struct WindowsOpenState {
    edit_map: bool,
    build: bool,
}

#[derive(Clone, Debug)]
pub struct UiConf {
    pub scale_factor: f32,
    pub font_scale: f32,
    pub max_message: usize,
}

impl Default for UiConf {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            font_scale: 1.4,
            max_message: 20,
        }
    }
}

#[derive(Clone, Default)]
pub struct UiTextures(HashMap<&'static str, (egui::TextureHandle, egui::Vec2)>);

pub const TEXTURE_LIST: &[&str] = &["ui/icon-branch.png", "ui/icon-build.png"];

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .add_startup_system(setup)
            .insert_resource(WindowsOpenState {
                edit_map: self.edit_map,
                ..default()
            })
            .init_resource::<UiConf>()
            .init_resource::<UiTextures>()
            .add_system(load_textures)
            .add_system(panels.label("ui_panels").before("ui_windows"))
            .add_system(build_window.label("ui_windows"))
            .add_system(edit_map_window.label("ui_windows"))
            .add_system(msg_window.label("ui_windows"))
            .add_system(exit_on_esc_system);
    }
}

fn setup(
    mut egui_ctx: ResMut<EguiContext>,
    mut egui_settings: ResMut<EguiSettings>,
    asset_server: Res<AssetServer>,
    conf: Res<UiConf>,
) {
    egui_settings.scale_factor = conf.scale_factor.into();

    let mut fonts = FontDefinitions::default();
    let mut font_data = FontData::from_static(include_bytes!("../fonts/Mplus2-SemiBold.otf"));
    font_data.tweak.scale = conf.font_scale;
    fonts.font_data.insert("m+_font".to_owned(), font_data);
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "m+_font".to_owned());
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .push("m+_font".to_owned());
    egui_ctx.ctx_mut().set_fonts(fonts);
    asset_server.load_folder("ui").unwrap();
}

fn exit_on_esc_system(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut _app_exit_events: EventWriter<AppExit>,
) {
    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ElementState::Pressed && key_code == KeyCode::Escape {
                std::process::exit(0);
                // app_exit_events.send(bevy::app::AppExit);
            }
        }
    }
}

fn load_textures(
    images: Res<Assets<Image>>,
    mut textures: ResMut<UiTextures>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    let ctx = egui_ctx.ctx_mut();

    for path in TEXTURE_LIST {
        if textures.0.contains_key(path) {
            continue;
        }
        let handle = images.get_handle(*path);
        if let Some(image) = images.get(handle) {
            let size = egui::Vec2::new(image.size().x, image.size().y);
            let color_image = egui::ColorImage {
                size: [size.x as usize, size.y as usize],
                pixels: image
                    .data
                    .windows(4)
                    .step_by(4)
                    .map(|rgba| {
                        egui::Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
                    })
                    .collect(),
            };
            let texture_handle = ctx.load_texture(*path, color_image);

            textures.0.insert(*path, (texture_handle, size));
        }
    }
}

fn panels(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    hover_tile: Query<&HoverTile>,
    mut cursor_mode: ResMut<CursorMode>,
    mut wos: ResMut<WindowsOpenState>,
    planet: Res<Planet>,
    textures: Res<UiTextures>,
    conf: Res<UiConf>,
) {
    occupied_screen_space.window_rects.clear();

    occupied_screen_space.occupied_left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            sidebar(ui, &cursor_mode, &planet, hover_tile.get_single().unwrap());
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width()
        * conf.scale_factor;

    occupied_screen_space.occupied_top = egui::TopBottomPanel::top("top_panel")
        .resizable(false)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                toolbar(ui, &mut cursor_mode, &mut wos, &textures, &conf);
            });
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height()
        * conf.scale_factor;
}

fn sidebar(ui: &mut egui::Ui, cursor_mode: &CursorMode, planet: &Planet, hover_tile: &HoverTile) {
    ui.label(&format!("{}: {}", t!("energy"), planet.player.energy));
    ui.label(&format!("{}: {}", t!("material"), planet.player.material));

    ui.separator();

    let s = if let Some(coords) = hover_tile.0 {
        format!("[{}, {}]", coords.0, coords.1)
    } else {
        "-".into()
    };
    ui.label(format!("{}: {}", t!("coordinates"), s));

    ui.separator();

    ui.label(t!("selected-tool"));

    match cursor_mode {
        CursorMode::Normal => {
            ui.label(t!("none"));
        }
        CursorMode::Build(kind) => match kind {
            StructureKind::Branch => {
                ui.label(t!("branch"));
            }
            _ => unreachable!(),
        },
        CursorMode::EditBiome(biome) => {
            ui.label(format!("biome editing: {}", biome.as_ref()));
        }
    }
}

fn toolbar(
    ui: &mut egui::Ui,
    cursor_mode: &mut CursorMode,
    wos: &mut WindowsOpenState,
    textures: &UiTextures,
    conf: &UiConf,
) {
    if let Some((handle, size)) = textures.0.get("ui/icon-branch.png") {
        if ui
            .add(
                egui::Button::image_and_text(handle.id(), conf.tex_size(*size), t!("branch"))
                    .small(),
            )
            .clicked()
        {
            *cursor_mode = CursorMode::Build(StructureKind::Branch);
        };
    }
    if let Some((handle, size)) = textures.0.get("ui/icon-build.png") {
        if ui
            .add(
                egui::Button::image_and_text(handle.id(), conf.tex_size(*size), t!("build"))
                    .small(),
            )
            .clicked()
        {
            wos.build = true;
        };
    }
}

fn build_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    conf: Res<UiConf>,
) {
    if !wos.build {
        return;
    }

    let rect = egui::Window::new("Select an item to build")
        .open(&mut wos.build)
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.label("工事中");
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

fn msg_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut msgs: Local<VecDeque<(MsgKind, String)>>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    conf: Res<UiConf>,
) {
    while let Some(msg) = crate::msg::pop_msg() {
        msgs.push_front(msg);
        if msgs.len() > conf.max_message {
            msgs.pop_back();
        }
    }

    let rect = egui::Window::new(t!("messages"))
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            for (_kind, msg) in msgs.iter() {
                ui.label(msg);
                ui.separator();
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

fn edit_map_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut cursor_mode: ResMut<CursorMode>,
    wos: Res<WindowsOpenState>,
    conf: Res<UiConf>,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    (mut new_w, mut new_h): (Local<u32>, Local<u32>),
    mut biome: Local<Biome>,
    mut save_file_path: Local<String>,
) {
    if !wos.edit_map {
        return;
    }

    let rect = egui::Window::new("Map editing tools")
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.add(egui::Slider::new(&mut *new_w, 2..=100).text("width"));
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut *new_h, 2..=100).text("height"));
                if ui.button("New").clicked() {
                    ew_manage_planet.send(ManagePlanet::New(*new_w, *new_h));
                }
            });

            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source(Biome::Ocean)
                    .selected_text(AsRef::<str>::as_ref(&*biome))
                    .show_ui(ui, |ui| {
                        use strum::IntoEnumIterator;
                        for b in Biome::iter() {
                            ui.selectable_value(&mut *biome, b, AsRef::<str>::as_ref(&b));
                        }
                    });
                if ui.button("Edit biome").clicked()
                    || matches!(*cursor_mode, CursorMode::EditBiome(_))
                {
                    *cursor_mode = CursorMode::EditBiome(*biome);
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut *save_file_path));
                if ui.button("Save").clicked() {
                    ew_manage_planet.send(ManagePlanet::Save(save_file_path.clone()));
                }
                if ui.button("Load").clicked() {
                    ew_manage_planet.send(ManagePlanet::Load(save_file_path.clone()));
                }
            });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

fn convert_rect(rect: bevy_egui::egui::Rect, scale_factor: f32) -> bevy::math::Rect<f32> {
    bevy::math::Rect {
        top: rect.top() * scale_factor,
        bottom: rect.bottom() * scale_factor,
        left: rect.left() * scale_factor,
        right: rect.right() * scale_factor,
    }
}

impl UiConf {
    fn tex_size(&self, size: egui::Vec2) -> egui::Vec2 {
        let factor = 1.0;
        egui::Vec2::new(size.x * factor, size.y * factor)
    }
}
