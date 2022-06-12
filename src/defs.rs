use geom::Coords;
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
pub struct StructureAttrs {
    #[serde(default)]
    pub size: StructureSize,
    pub width: u32,
    pub height: u32,
    pub columns: usize,
    pub rows: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StructureSize {
    Small,
    Middle,
}

impl StructureSize {
    /// Additional occupiied tiles by a structure
    pub fn occupied_tiles(&self) -> Vec<Coords> {
        match self {
            StructureSize::Small => vec![],
            StructureSize::Middle => vec![Coords(1, 0), Coords(1, 1), Coords(0, 1)],
        }
    }
}

impl Default for StructureSize {
    fn default() -> Self {
        Self::Small
    }
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
#[strum_discriminants(derive(Hash, Serialize, Deserialize, EnumIter, AsRefStr))]
#[strum_discriminants(serde(rename_all = "kebab-case"))]
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
