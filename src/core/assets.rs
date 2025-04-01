use crate::core::ants::components::Ant;
use bevy::asset::{AssetServer, Handle};
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Clone)]
pub struct TextureInfo {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Clone)]
pub struct AtlasInfo {
    pub image: Handle<Image>,
    pub texture: TextureAtlas,
    pub last_index: usize,
}

pub struct WorldAssets {
    pub audio: HashMap<&'static str, Handle<AudioSource>>,
    pub fonts: HashMap<&'static str, Handle<Font>>,
    pub images: HashMap<&'static str, Handle<Image>>,
    pub textures: HashMap<&'static str, TextureInfo>,
    pub atlas: HashMap<&'static str, AtlasInfo>,
}

impl WorldAssets {
    fn get_asset<'a, T: Clone>(
        &self,
        map: &'a HashMap<&str, T>,
        name: &str,
        asset_type: &str,
    ) -> &'a T {
        map.get(name)
            .expect(&format!("No asset for {asset_type} {name}"))
    }

    pub fn audio(&self, name: &str) -> Handle<AudioSource> {
        self.get_asset(&self.audio, name, "audio").clone_weak()
    }

    pub fn font(&self, name: &str) -> Handle<Font> {
        self.get_asset(&self.fonts, name, "font").clone_weak()
    }

    pub fn image(&self, name: &str) -> Handle<Image> {
        self.get_asset(&self.images, name, "image").clone_weak()
    }

    pub fn texture(&self, name: &str) -> TextureInfo {
        self.get_asset(&self.textures, name, "texture").clone()
    }

    pub fn atlas(&self, name: &str) -> AtlasInfo {
        self.get_asset(&self.atlas, name, "atlas").clone()
    }
}

impl FromWorld for WorldAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();

        let audio = HashMap::from([
            ("button", assets.load("audio/button.ogg")),
            ("message", assets.load("audio/message.ogg")),
            ("warning", assets.load("audio/warning.ogg")),
            ("error", assets.load("audio/error.ogg")),
            ("defeat", assets.load("audio/defeat.ogg")),
            ("music", assets.load("audio/music.ogg")),
        ]);

        let fonts = HashMap::from([
            ("bold", assets.load("fonts/FiraSans-Bold.ttf")),
            ("medium", assets.load("fonts/FiraMono-Medium.ttf")),
        ]);

        let mut images: HashMap<&'static str, Handle<Image>> = HashMap::from([
            // Icons
            ("mute", assets.load("images/icons/mute.png")),
            ("no-music", assets.load("images/icons/no-music.png")),
            ("sound", assets.load("images/icons/sound.png")),
            ("pin", assets.load("images/icons/pin.png")),
            ("attack", assets.load("images/icons/attack.png")),
            ("defend", assets.load("images/icons/defend.png")),
            // Map
            ("defeat", assets.load("images/map/defeat.png")),
            ("victory", assets.load("images/map/victory.png")),
            ("tiles", assets.load("images/map/soil_tileset.png")),
            ("base", assets.load("images/map/base.png")),
            ("food", assets.load("images/map/food.png")),
            ("nutrient", assets.load("images/map/nutrient.png")),
            ("blood", assets.load("images/map/blood.png")),
            ("hole1", assets.load("images/map/soil_hole_01.png")),
            ("hole2", assets.load("images/map/soil_hole_02.png")),
            ("seed1", assets.load("images/map/seed_01.png")),
            ("seed2", assets.load("images/map/seed_02.png")),
            ("seed3", assets.load("images/map/seed_03.png")),
            ("stick1", assets.load("images/map/stick_01.png")),
            ("stick2", assets.load("images/map/stick_02.png")),
            ("stick3", assets.load("images/map/stick_03.png")),
            ("leaf1", assets.load("images/map/leaf_01.png")),
            ("leaf2", assets.load("images/map/leaf_02.png")),
            ("leaf3", assets.load("images/map/leaf_03.png")),
            ("leaf4", assets.load("images/map/leaf_04.png")),
            ("leaf5", assets.load("images/map/leaf_05.png")),
            ("stone1", assets.load("images/map/stone_01.png")),
            ("stone2", assets.load("images/map/stone_02.png")),
            ("stone3", assets.load("images/map/stone_03.png")),
            ("stone4", assets.load("images/map/stone_04.png")),
            ("stone5", assets.load("images/map/stone_05.png")),
            ("stone6", assets.load("images/map/stone_06.png")),
            ("stone7", assets.load("images/map/stone_07.png")),
            ("stone8", assets.load("images/map/stone_08.png")),
            ("stone9", assets.load("images/map/stone_09.png")),
            ("stone10", assets.load("images/map/stone_10.png")),
            ("stone11", assets.load("images/map/stone_11.png")),
            ("stone12", assets.load("images/map/stone_12.png")),
            ("stone13", assets.load("images/map/stone_13.png")),
            ("stone14", assets.load("images/map/stone_14.png")),
            ("stone15", assets.load("images/map/stone_15.png")),
            ("stone16", assets.load("images/map/stone_16.png")),
            ("stone17", assets.load("images/map/stone_17.png")),
            ("stone18", assets.load("images/map/stone_18.png")),
            ("egg", assets.load("images/map/egg.png")),
            ("larva1", assets.load("images/map/larva_01.png")),
            ("larva2", assets.load("images/map/larva_02.png")),
            // Traits
            ("alate", assets.load("images/traits/alate.png")),
            ("battle", assets.load("images/traits/battle.png")),
            ("cannibal", assets.load("images/traits/cannibal.png")),
            ("corpses", assets.load("images/traits/corpses.png")),
            ("double-queen", assets.load("images/traits/two-queens.png")),
            ("eggs", assets.load("images/traits/eggs.png")),
            ("harvest", assets.load("images/traits/harvest.png")),
            ("haste", assets.load("images/traits/haste.png")),
            ("healing", assets.load("images/traits/healing.png")),
            ("hole", assets.load("images/traits/hole.png")),
            ("influx", assets.load("images/traits/influx.png")),
            ("mastodon", assets.load("images/traits/mastodon.png")),
            ("megacolony", assets.load("images/traits/megacolony.png")),
            (
                "metamorfosis",
                assets.load("images/traits/metamorfosis.png"),
            ),
            ("necromancer", assets.load("images/traits/necromancer.png")),
            ("scorpion", assets.load("images/traits/scorpion.png")),
            ("soldiers", assets.load("images/traits/soldiers.png")),
            ("sudden-army", assets.load("images/traits/sudden-army.png")),
            ("super-queen", assets.load("images/traits/super-queen.png")),
            ("termites", assets.load("images/traits/termites.png")),
            ("tunneling", assets.load("images/traits/tunneling.png")),
            ("wandering", assets.load("images/traits/wandering.png")),
            ("wasp", assets.load("images/traits/wasp.png")),
            ("workers", assets.load("images/traits/workers.png")),
        ]);

        for ant in Ant::iter() {
            for animation in ant.all_animations() {
                for ant_c in ant.colors() {
                    let name = Box::leak(Box::new(ant_c.atlas(&animation))).as_str();

                    images.insert(
                        name,
                        assets.load(&format!("images/ants/{}/{}.png", ant_c.folder(), name)),
                    );
                }
            }
        }

        let mut texture = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .unwrap();

        let tiles = TextureAtlasLayout::from_grid(UVec2::splat(128), 8, 9, None, None);

        let textures: HashMap<&'static str, TextureInfo> = HashMap::from([(
            "tiles",
            TextureInfo {
                image: images["tiles"].clone_weak(),
                layout: texture.add(tiles),
            },
        )]);

        let mut atlas = HashMap::new();

        for ant in Ant::iter() {
            for animation in ant.all_animations() {
                for ant_c in ant.colors() {
                    let name = Box::leak(Box::new(ant_c.atlas(&animation))).as_str();

                    let layout = TextureAtlasLayout::from_grid(
                        ant_c.size().as_uvec2(),
                        ant.frames(&animation),
                        1,
                        None,
                        None,
                    );

                    atlas.insert(
                        name,
                        AtlasInfo {
                            image: images[name].clone_weak(),
                            texture: TextureAtlas {
                                layout: texture.add(layout),
                                index: 0,
                            },
                            last_index: ant.frames(&animation) as usize - 1,
                        },
                    );
                }
            }
        }

        Self {
            audio,
            fonts,
            images,
            textures,
            atlas,
        }
    }
}
