use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumDiscriminants, EnumIter, EnumString};

pub const TILE_SIZE: f32 = 48.0;
pub const PIECE_SIZE: f32 = TILE_SIZE / 2.0;

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    EnumString,
    EnumIter,
    AsRefStr,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Biome {
    Ocean,
    Mountains,
    Desert,
    Grassland,
}

impl Default for Biome {
    fn default() -> Self {
        Self::Ocean
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiomeAttrs {
    pub z: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LandFeature {
    None,
    Oil,
    Lime,
    Iron,
}

#[derive(Clone, Debug, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(name(StructureKind))]
#[strum_discriminants(derive(Hash, EnumIter, AsRefStr))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum Structure {
    None,
    Occupied,
    Branch,
    Core,
    GathererDroneHub,
    CombatDroneHub,
    PhotosynthesisModule,
    SiliconChemModule,
    MiningModule,
}
