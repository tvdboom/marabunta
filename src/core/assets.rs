use bevy::asset::{AssetServer, Handle};
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use std::collections::HashMap;

pub struct WorldAssets {
    pub images: HashMap<&'static str, Handle<Image>>,
    pub audio: HashMap<&'static str, Handle<AudioSource>>,
}

impl WorldAssets {
    pub fn image(&self, name: &str) -> Handle<Image> {
        self.images
            .get(name)
            .expect(&format!("No asset for image {name}"))
            .clone_weak()
    }

    pub fn audio(&self, name: &str) -> Handle<AudioSource> {
        self.audio
            .get(name)
            .expect(&format!("No asset for audio {name}"))
            .clone_weak()
    }
}

impl FromWorld for WorldAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();

        let images: HashMap<&'static str, Handle<Image>> = HashMap::from([
            ("mute", assets.load("icons/mute.png")),
            ("sound", assets.load("icons/sound.png")),
        ]);

        let audio = HashMap::from([("music", assets.load("audio/music.ogg"))]);

        Self { images, audio }
    }
}
