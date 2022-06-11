use crate::defs::*;
use geom::Array2d;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub biome: Biome,
    pub land_feature: LandFeature,
    pub structure: Structure,
    pub biomass: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub map: Array2d<Tile>,
}

impl Planet {
    pub fn new(w: u32, h: u32) -> Planet {
        let map = Array2d::new(w, h, Tile::default());

        Planet { tick: 0, map }
    }
}
