use bevy::prelude::*;
use geom::Coords;

use crate::defs::*;
use crate::planet::Planet;
use crate::screen::CursorMode;

#[derive(Clone, Copy, Debug)]
pub struct ActionPlugin;

#[derive(Clone, Copy, Debug)]
pub struct CursorAction {
    pub coords: Coords,
    pub drag: bool,
}

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CursorAction>().add_system(cursor_action);
    }
}

fn cursor_action(
    mut er: EventReader<CursorAction>,
    cursor_mode: Res<CursorMode>,
    mut planet: ResMut<Planet>,
) {
    for e in er.iter() {
        let CursorAction { coords, .. } = *e;

        match *cursor_mode {
            CursorMode::Normal => {
                println!("{}", coords);
            }
            CursorMode::EditBiome(idx) => {
                planet.map[coords].biome = idx;
            }
            CursorMode::Build(kind) => match kind {
                StructureKind::None => (),
                StructureKind::Branch => {
                    if matches!(planet.map[coords].structure, Structure::None) {
                        planet.map[coords].structure = Structure::Branch;
                    }
                }
                _ => todo!(),
            },
        }
    }
}
