use crate::assets::*;
use crate::defs::*;
use crate::planet::*;
use crate::screen::InScreenTileRange;
use arrayvec::ArrayVec;
use bevy::{core::FixedTimestep, prelude::*};
use geom::{Array2d, Coords, Direction, RectIter};

#[derive(Clone, Copy, Debug)]
pub struct DrawPlugin;

pub const DRAW_FPS: f64 = 30.0;

const CORNERS: [Coords; 4] = [Coords(-1, -1), Coords(-1, 1), Coords(1, 1), Coords(1, -1)];

const CORNER_PIECE_GRID: [(usize, usize); 4] = [(0, 1), (0, 0), (1, 0), (1, 1)];

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(initial_tile_world)
            .add_system(update_layered_tex_map.label("draw"))
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::steps_per_second(DRAW_FPS))
                    .with_system(spawn_map_textures.label("draw")),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::steps_per_second(DRAW_FPS))
                    .with_system(spawn_structure_textures.label("draw")),
            );
    }
}

pub struct LayeredTexMap {
    biome: Array2d<ArrayVec<Biome, 9>>,
}

fn initial_tile_world(mut commands: Commands) {
    let planet = Planet::new(30, 30);

    commands.insert_resource(planet);
}

fn update_layered_tex_map(
    mut commands: Commands,
    assets: Option<Res<AssetsLoaded>>,
    planet: Res<Planet>,
    ltm: Option<ResMut<LayeredTexMap>>,
) {
    if !planet.is_changed() && ltm.is_some() {
        return;
    }
    let assets = if let Some(assets) = &assets {
        assets
    } else {
        return;
    };

    let (w, h) = planet.map.size();
    let mut tiles = Array2d::new(w, h, ArrayVec::new());

    for &i in assets.biomes.keys() {
        for pos in RectIter::new((0, 0), (w - 1, h - 1)) {
            let biome_i = planet.map[pos].biome;
            if biome_i != i {
                continue;
            }

            let tile_z = assets.biomes[&biome_i].attrs.z;
            tiles[pos].push(i);
            for d in Direction::EIGHT_DIRS {
                let p = pos + d.as_coords();
                if tiles.in_range(p) {
                    let surround_tile_i = planet.map[p].biome;
                    let z = assets.biomes[&surround_tile_i].attrs.z;
                    if z < tile_z && !tiles[pos].contains(&surround_tile_i) {
                        tiles[pos].push(surround_tile_i);
                    }
                }
            }
        }
    }

    let ltm = LayeredTexMap { biome: tiles };
    commands.insert_resource(ltm);
}

fn spawn_map_textures(
    mut commands: Commands,
    ltm: Option<Res<LayeredTexMap>>,
    assets: Option<Res<AssetsLoaded>>,
    in_screen_tile_range: ResMut<InScreenTileRange>,
    mut tex_entities: Local<Vec<Entity>>,
) {
    let (ltm, assets) = if let (Some(ltm), Some(assets)) = (&ltm, &assets) {
        (ltm, assets)
    } else {
        return;
    };

    for entity in tex_entities.iter() {
        commands.entity(*entity).despawn();
    }
    tex_entities.clear();

    // Spawn biome textures
    for p in RectIter::new(in_screen_tile_range.from, in_screen_tile_range.to) {
        for tile_idx in &ltm.biome[p] {
            for (corner, corner_piece_grid) in CORNERS.into_iter().zip(CORNER_PIECE_GRID) {
                let corner_index = corner_idx(
                    |pos| {
                        if ltm.biome.in_range(pos) {
                            ltm.biome[pos].contains(tile_idx)
                        } else {
                            true
                        }
                    },
                    p,
                    corner,
                );

                let grid_x = (corner_index % 3) * 2 + corner_piece_grid.0;
                let grid_y = (corner_index / 3) * 2 + corner_piece_grid.1;

                let index = grid_x + grid_y * 6;

                let sprite = TextureAtlasSprite { index, ..default() };

                let x = p.0 as f32 * TILE_SIZE
                    + PIECE_SIZE * ((corner.0 + 1) / 2) as f32
                    + PIECE_SIZE / 2.0;
                let y = p.1 as f32 * TILE_SIZE
                    + PIECE_SIZE * ((corner.1 + 1) / 2) as f32
                    + PIECE_SIZE / 2.0;

                let tile_asset = &assets.biomes[tile_idx];
                let id = commands
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: tile_asset.texture_atlas.clone(),
                        sprite,
                        transform: Transform::from_xyz(x, y, tile_asset.attrs.z / 10.0),
                        visibility: Visibility { is_visible: true },
                        ..default()
                    })
                    .id();
                tex_entities.push(id);
            }
        }
    }
}

fn spawn_structure_textures(
    mut commands: Commands,
    assets: Option<Res<AssetsLoaded>>,
    in_screen_tile_range: ResMut<InScreenTileRange>,
    planet: Res<Planet>,
    mut tex_entities: Local<Vec<Entity>>,
) {
    let assets = if let Some(assets) = &assets {
        assets
    } else {
        return;
    };
    for entity in tex_entities.iter() {
        commands.entity(*entity).despawn();
    }
    tex_entities.clear();

    for p in RectIter::new(in_screen_tile_range.from, in_screen_tile_range.to) {
        let structure = &planet.map[p].structure;

        match structure {
            Structure::None => (),
            Structure::Branch => {
                for (corner, corner_piece_grid) in CORNERS.into_iter().zip(CORNER_PIECE_GRID) {
                    let corner_index = corner_idx(
                        |pos| {
                            if planet.map.in_range(pos) {
                                matches!(planet.map[pos].structure, Structure::Branch)
                            } else {
                                false
                            }
                        },
                        p,
                        corner,
                    );

                    let grid_x = (corner_index % 3) * 2 + corner_piece_grid.0;
                    let grid_y = (corner_index / 3) * 2 + corner_piece_grid.1;

                    let index = grid_x + grid_y * 6;

                    let sprite = TextureAtlasSprite { index, ..default() };

                    let x = p.0 as f32 * TILE_SIZE
                        + PIECE_SIZE * ((corner.0 + 1) / 2) as f32
                        + PIECE_SIZE / 2.0;
                    let y = p.1 as f32 * TILE_SIZE
                        + PIECE_SIZE * ((corner.1 + 1) / 2) as f32
                        + PIECE_SIZE / 2.0;

                    // let asset = &assets.biomes[&Biome::Desert];
                    let asset = &assets.structures[&StructureKind::Branch];

                    let id = commands
                        .spawn_bundle(SpriteSheetBundle {
                            texture_atlas: asset.texture_atlas.clone(),
                            sprite,
                            transform: Transform::from_xyz(x, y, 300.0),
                            visibility: Visibility { is_visible: true },
                            ..default()
                        })
                        .id();
                    tex_entities.push(id);
                }
            }
            _ => todo!(),
        }
    }
}

fn corner_idx<F: Fn(Coords) -> bool>(f: F, pos: Coords, corner: Coords) -> usize {
    let a = f(pos + (corner.0, 0));
    let b = f(pos + (0, corner.1));
    let c = f(pos + corner);

    match (a, b, c) {
        (true, true, true) => 0,
        (true, false, _) => 1,
        (false, true, _) => 2,
        (false, false, _) => 3,
        (true, true, false) => 4,
    }
}
