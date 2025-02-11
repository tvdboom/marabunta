use bevy::asset::{AssetServer, Handle};
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use std::collections::HashMap;

#[derive(Clone)]
pub struct TextureInfo {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

pub struct WorldAssets {
    pub audio: HashMap<&'static str, Handle<AudioSource>>,
    pub fonts: HashMap<&'static str, Handle<Font>>,
    pub images: HashMap<&'static str, Handle<Image>>,
    pub textures: HashMap<&'static str, TextureInfo>,
}

impl WorldAssets {
    pub fn image(&self, name: &str) -> Handle<Image> {
        self.images
            .get(name)
            .expect(&format!("No asset for image {name}"))
            .clone_weak()
    }

    pub fn texture(&self, name: &str) -> TextureInfo {
        self.textures
            .get(name)
            .expect(&format!("No asset for texture {name}"))
            .clone()
    }

    pub fn audio(&self, name: &str) -> Handle<AudioSource> {
        self.audio
            .get(name)
            .expect(&format!("No asset for audio {name}"))
            .clone_weak()
    }

    pub fn font(&self, name: &str) -> Handle<Font> {
        self.fonts
            .get(name)
            .expect(&format!("No asset for font {name}"))
            .clone_weak()
    }
}

impl FromWorld for WorldAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();

        let audio = HashMap::from([
            ("button", assets.load("audio/button.ogg")),
            ("music", assets.load("audio/music.ogg")),
        ]);

        let fonts = HashMap::from([
            ("FiraSans-Bold", assets.load("fonts/FiraSans-Bold.ttf")),
            ("FiraMono-Medium", assets.load("fonts/FiraMono-Medium.ttf")),
        ]);

        let images: HashMap<&'static str, Handle<Image>> = HashMap::from([
            // Icons
            ("mute", assets.load("images/icons/mute.png")),
            ("sound", assets.load("images/icons/sound.png")),
            // Map
            ("tiles", assets.load("images/map/soil_tileset.png")),
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
        ]);

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

        Self {
            audio,
            fonts,
            images,
            textures,
        }
    }
}
