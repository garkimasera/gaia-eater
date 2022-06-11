use crate::action::CursorAction;
use crate::defs::{Biome, StructureKind, TILE_SIZE};
use crate::planet::Planet;
use bevy::math::{Rect, Vec3Swizzles};
use bevy::prelude::*;
use geom::Coords;

#[derive(Clone, Copy, Debug)]
pub struct ScreenPlugin;

#[derive(Clone, Copy, Debug)]
pub struct Centering(pub Vec2);

#[derive(Clone, Debug)]
pub enum CursorMode {
    Normal,
    EditBiome(Biome),
    Build(StructureKind),
}

impl Default for CursorMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Centering>()
            .add_startup_system(setup)
            .init_resource::<OccupiedScreenSpace>()
            .init_resource::<InScreenTileRange>()
            .init_resource::<HoverTile>()
            .init_resource::<CursorMode>()
            .add_system(centering.before("draw"))
            .add_system(
                update_hover_tile
                    .label("update_hover_tile")
                    .after("ui_windows"),
            )
            .add_system(mouse_event.after("update_hover_tile"));
    }
}

#[derive(Clone, Debug)]
pub struct InScreenTileRange {
    pub from: Coords,
    pub to: Coords,
}

#[derive(Clone, Copy, Default, Debug, Component)]
pub struct HoverTile(pub Option<Coords>);

impl Default for InScreenTileRange {
    fn default() -> Self {
        Self {
            from: Coords(0, 0),
            to: Coords(0, 0),
        }
    }
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera = OrthographicCameraBundle::new_2d();
    commands.spawn_bundle(camera);

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("ui/tile-cursor.png"),
            visibility: Visibility { is_visible: false },
            ..default()
        })
        .insert(HoverTile(None));
}

fn mouse_event(
    mut ew_cursor_action: EventWriter<CursorAction>,
    mut ew_centering: EventWriter<Centering>,
    windows: Res<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    camera_query: Query<(&OrthographicProjection, &Transform)>,
    occupied_screen_space: Res<OccupiedScreenSpace>,
    hover_tile: Query<(&HoverTile, &Transform), Without<OrthographicProjection>>,
    mut prev_tile_coords: Local<Option<Coords>>,
) {
    let window = windows.get_primary().unwrap();
    let pos = if let Some(pos) = window.cursor_position() {
        pos
    } else {
        return;
    };

    if !occupied_screen_space.check(window.width(), window.height(), pos) {
        return;
    }

    // Centering
    if mouse_button_input.just_pressed(MouseButton::Middle) {
        let transform = camera_query.get_single().unwrap().1;
        let mut translation = transform.translation.xy();

        let d = Vec2::new(pos.x - window.width() / 2.0, pos.y - window.height() / 2.0);

        translation += d;

        ew_centering.send(Centering(translation));
        return;
    }

    // Cursor action
    if !mouse_button_input.pressed(MouseButton::Left) {
        *prev_tile_coords = None;
        return;
    }

    if let Some(coords) = hover_tile.get_single().unwrap().0 .0 {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            ew_cursor_action.send(CursorAction {
                coords,
                drag: false,
            });
            *prev_tile_coords = Some(coords);
            return;
        }

        if prev_tile_coords.is_some() && Some(coords) != *prev_tile_coords {
            ew_cursor_action.send(CursorAction { coords, drag: true });
            *prev_tile_coords = Some(coords);
        }
    }
}

fn centering(
    mut er_centering: EventReader<Centering>,
    screen: Res<OccupiedScreenSpace>,
    windows: Res<Windows>,
    egui_settings: ResMut<bevy_egui::EguiSettings>,
    mut in_screen_tile_range: ResMut<InScreenTileRange>,
    planet: Res<Planet>,
    mut camera_query: Query<(&OrthographicProjection, &mut Transform)>,
) {
    let transform = &mut camera_query.get_single_mut().unwrap().1.translation;
    let window = windows.get_primary().unwrap();

    for e in er_centering.iter() {
        let center = &e.0;

        // Change camera position
        transform.x = center
            .x
            .clamp(-TILE_SIZE, (planet.map.size().0 + 1) as f32 * TILE_SIZE);
        transform.y = center
            .y
            .clamp(-TILE_SIZE, (planet.map.size().1 + 1) as f32 * TILE_SIZE);

        let space_adjust = Vec3::new(
            (screen.occupied_left - screen.occupied_right) * egui_settings.scale_factor as f32,
            (screen.occupied_buttom - screen.occupied_top) * egui_settings.scale_factor as f32,
            0.0,
        ) / 2.0;
        *transform -= space_adjust;

        transform.x = transform.x.round();
        transform.y = transform.y.round();

        // Update in screnn tile range
        let x0 = (((transform.x - window.width() / 2.0) / TILE_SIZE) as i32 - 1)
            .clamp(0, planet.map.size().0 as i32 - 1);
        let y0 = (((transform.y - window.height() / 2.0) / TILE_SIZE) as i32 - 1)
            .clamp(0, planet.map.size().1 as i32 - 1);
        let x1 = (((transform.x + window.width() / 2.0) / TILE_SIZE) as i32 + 1)
            .clamp(0, planet.map.size().0 as i32 - 1);
        let y1 = (((transform.y + window.height() / 2.0) / TILE_SIZE) as i32 + 1)
            .clamp(0, planet.map.size().1 as i32 - 1);
        in_screen_tile_range.from = Coords(x0, y0);
        in_screen_tile_range.to = Coords(x1, y1);
    }
}

fn update_hover_tile(
    windows: Res<Windows>,
    planet: Res<Planet>,
    mut hover_tile: Query<
        (&mut HoverTile, &mut Transform, &mut Visibility),
        Without<OrthographicProjection>,
    >,
    camera_query: Query<(&OrthographicProjection, &Transform)>,
) {
    let mut hover_tile = hover_tile.get_single_mut().unwrap();
    let window = windows.get_primary().unwrap();
    let cursor_pos = if let Some(pos) = window.cursor_position() {
        pos
    } else {
        return;
    };

    let camera_pos = camera_query.get_single().unwrap().1.translation.xy();

    let p = cursor_pos + camera_pos - Vec2::new(window.width() / 2.0, window.height() / 2.0);

    let tile_i = (p.x / TILE_SIZE) as i32;
    let tile_j = (p.y / TILE_SIZE) as i32;

    if tile_i >= 0
        && tile_i < planet.map.size().0 as i32
        && tile_j >= 0
        && tile_j < planet.map.size().1 as i32
        && p.x >= 0.0
        && p.y >= 0.0
    {
        hover_tile.0 .0 = Some(Coords(tile_i, tile_j));
        hover_tile.1.translation.x = tile_i as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        hover_tile.1.translation.y = tile_j as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        hover_tile.1.translation.z = 900.0;
        *hover_tile.2 = Visibility { is_visible: true };
    } else {
        hover_tile.0 .0 = None;
        *hover_tile.2 = Visibility { is_visible: false };
    };
}

#[derive(Clone, Default, Debug)]
pub struct OccupiedScreenSpace {
    pub occupied_top: f32,
    pub occupied_buttom: f32,
    pub occupied_left: f32,
    pub occupied_right: f32,
    pub window_rects: Vec<Rect<f32>>,
}

impl OccupiedScreenSpace {
    fn check(&self, w: f32, h: f32, p: Vec2) -> bool {
        if p.x < self.occupied_left
            || p.x > w - self.occupied_right
            || p.y < self.occupied_buttom
            || p.y > h - self.occupied_top
        {
            return false;
        }

        let x = p.x;
        let y = h - p.y;

        for rect in &self.window_rects {
            if rect.left <= x && x <= rect.right && rect.top <= y && y <= rect.bottom {
                return false;
            }
        }

        true
    }
}
