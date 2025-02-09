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
