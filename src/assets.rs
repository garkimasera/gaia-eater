use crate::defs::*;
use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use fnv::FnvHashMap;
use serde::Deserialize;

#[derive(Clone, Copy, Debug)]
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_asset_ron::RonAssetPlugin::<BiomeAssetList>::new(&[
            "biomes.ron",
        ]))
        .add_plugin(bevy_asset_ron::RonAssetPlugin::<StructureAssetList>::new(
            &["structures.ron"],
        ))
        .init_resource::<AssetsLoading>()
        .add_startup_system(load_assets)
        .add_system(create_assets_list);
    }
}

#[derive(Default)]
struct AssetsLoading(Vec<HandleUntyped>);

#[derive(Clone, Debug, Deserialize, TypeUuid)]
#[serde(transparent)]
#[uuid = "99d5021f-98fb-4873-b16a-bd9619b8b074"]
pub struct BiomeAssetList(FnvHashMap<Biome, BiomeAttrs>);

#[derive(Clone, Debug, Deserialize, TypeUuid)]
#[serde(transparent)]
#[uuid = "801a2daa-956d-469a-8e83-3610fbca21fd"]
pub struct StructureAssetList(FnvHashMap<StructureKind, StructureAttrs>);

pub struct AssetsLoaded {
    pub biomes: FnvHashMap<Biome, BiomeAsset>,
    pub structures: FnvHashMap<StructureKind, StructureAsset>,
}

pub struct BiomeAsset {
    pub attrs: BiomeAttrs,
    pub texture_atlas: Handle<TextureAtlas>,
}

pub struct StructureAsset {
    pub attrs: StructureAttrs,
    pub texture_atlas: Handle<TextureAtlas>,
}

fn load_assets(asset_server: Res<AssetServer>, mut assets_loading: ResMut<AssetsLoading>) {
    assets_loading
        .0
        .append(&mut asset_server.load_folder("biomes").unwrap());
    assets_loading
        .0
        .append(&mut asset_server.load_folder("structures").unwrap());
}

fn create_assets_list(
    mut command: Commands,
    asset_server: Res<AssetServer>,
    loading: Option<Res<AssetsLoading>>,
    biomes: Res<Assets<BiomeAssetList>>,
    structures: Res<Assets<StructureAssetList>>,
    images: Res<Assets<Image>>,
    mut texture_atlas_assets: ResMut<Assets<TextureAtlas>>,
) {
    let loading = if let Some(loading) = loading {
        loading
    } else {
        return;
    };

    match asset_server.get_group_load_state(loading.0.iter().map(|h| h.id)) {
        LoadState::Failed => {
            panic!();
        }
        LoadState::Loaded => (),
        _ => {
            return;
        }
    }

    let biomes = biomes
        .iter()
        .next()
        .expect("biomes not found")
        .1
         .0
        .iter()
        .map(|(biome, attrs)| {
            let image = images.get_handle(&format!("biomes/{}.png", AsRef::<str>::as_ref(biome)));
            let texture_atlas =
                TextureAtlas::from_grid(image, Vec2::new(PIECE_SIZE, PIECE_SIZE), 6, 4);
            let texture_atlas = texture_atlas_assets.add(texture_atlas);

            (
                *biome,
                BiomeAsset {
                    attrs: attrs.clone(),
                    texture_atlas,
                },
            )
        })
        .collect();

    let structures = structures
        .iter()
        .next()
        .expect("structures not found")
        .1
         .0
        .iter()
        .map(|(structure, attrs)| {
            let image = images.get_handle(&format!(
                "structures/{}.png",
                AsRef::<str>::as_ref(structure)
            ));
            let texture_atlas = TextureAtlas::from_grid(
                image,
                Vec2::new(attrs.width as _, attrs.height as _),
                attrs.columns,
                attrs.rows,
            );
            let texture_atlas = texture_atlas_assets.add(texture_atlas);

            (
                *structure,
                StructureAsset {
                    attrs: attrs.clone(),
                    texture_atlas,
                },
            )
        })
        .collect();

    command.insert_resource(AssetsLoaded { biomes, structures });
    command.remove_resource::<AssetsLoading>();
}
