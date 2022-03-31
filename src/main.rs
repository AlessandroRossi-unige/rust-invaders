#![allow(unused)]


use std::path::Path;
use bevy::prelude::*;
use bevy::render::texture::ImageType;
use crate::StartupStage::Startup;

const SPRITE_DIR: &str = "assets";
pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);

const PLAYER_SPRITE: &str = "player_a_01.png";

pub struct SpriteInfos {
    player: (Handle<Image>, Vec2)
}

struct WinSize {
    w: f32,
    h: f32,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Rust Invaders!".to_string(),
            width: 598.0,
            height: 676.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(player_spawn.system())
        .run();
}


fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut windows: ResMut<Windows>
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let mut window = windows.get_primary_mut().unwrap();

    commands.insert_resource(WinSize{
        w: window.width(),
        h: window.height()
    });

    window.set_position(IVec2::new(1300, 80));

    commands.insert_resource(SpriteInfos {
        player: load_image(&mut images, PLAYER_SPRITE)
    });

}

fn load_image(images: &mut ResMut<Assets<Image>>, path: &str) -> (Handle<Image>, Vec2) {
    let path = Path::new(SPRITE_DIR).join(path);
    let bytes = std::fs::read(&path).expect(&format!("Cannot find {}", path.display()));
    let image = Image::from_buffer(&bytes, ImageType::MimeType("image/png")).unwrap();
    let size = image.texture_descriptor.size;
    let size = Vec2::new(size.width as f32, size.height as f32);
    let image_handle = images.add(image);
    (image_handle, size)
}


fn player_spawn(mut commands: Commands, asset_server: Res<AssetServer>, win_size: Res<WinSize>) {
    let bottom = win_size.h / 2.;

    let image = asset_server.load(PLAYER_SPRITE);
    commands.spawn_bundle(SpriteBundle {
        texture: image,
        transform: Transform {
            translation: Vec3::new(0., -bottom + 25., 10.),
            scale: Vec3::new(0.5, 0.5, 0.1),
            ..Default::default()
        },
        ..Default::default()
    });
}