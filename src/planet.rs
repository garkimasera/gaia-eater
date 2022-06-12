use crate::defs::*;
use geom::{Array2d, Coords};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub biome: Biome,
    pub land_feature: LandFeature,
    pub structure: Structure,
    pub biomass: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Player {
    pub energy: f32,
    pub material: f32,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            biome: Biome::Ocean,
            land_feature: LandFeature::None,
            structure: Structure::None,
            biomass: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Planet {
    pub tick: u64,
    pub player: Player,
    pub map: Array2d<Tile>,
}

impl Planet {
    pub fn new(w: u32, h: u32) -> Planet {
        let map = Array2d::new(w, h, Tile::default());

        let mut planet = Planet {
            tick: 0,
            player: Player::default(),
            map,
        };

        planet.place(
            (w / 2 - 1, h / 2 - 1).into(),
            StructureSize::Middle,
            Structure::Core,
        );

        planet
    }

    pub fn placeable(&self, p: Coords, size: StructureSize) -> bool {
        if !self.map.in_range(p) {
            return false;
        }

        for p in size.occupied_tiles().into_iter() {
            if let Some(tile) = self.map.get(p) {
                if !matches!(tile.structure, Structure::None) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn place(&mut self, p: Coords, size: StructureSize, structure: Structure) {
        assert!(self.placeable(p, size));

        self.map[p].structure = structure;

        for p_rel in size.occupied_tiles().into_iter() {
            self.map[p + p_rel].structure = Structure::Occupied { by: p };
        }
    }
}
