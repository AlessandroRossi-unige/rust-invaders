#![allow(unused)]


use std::borrow::BorrowMut;
use std::path::Path;
use std::process::exit;
use std::ptr::addr_of;
use bevy::app::AppExit;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::texture::ImageType;
use bevy::window::CloseWindow;
use crate::StartupStage::Startup;

const SPRITE_DIR: &str = "assets";

const PLAYER_SPRITE: &str = "player_a_01.png";
const LASER_SPRITE: &str = "laser_a_01.png";

const TIME_STEP: f32 = 1. / 60.;

pub struct SpriteInfos {
    player: (Handle<Image>, Vec2),
    laser: (Handle<Image>, Vec2)
}

struct WinSize {
    w: f32,
    h: f32,
}


pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_stage(
                "game_setup_actors",
                SystemStage::single(player_spawn.system()),
            )
            .add_system(player_movement.system())
            .add_system(player_fire.system())
            .add_system(laser_movement.system());

    }
}

#[derive(Component)]
struct Player;
#[derive(Component)]
struct Laser;
#[derive(Component)]
struct Speed(f32);

impl Default for Speed {
    fn default() -> Self {
        Self(500.)
    }
}

fn main() {
    App::new()

        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(WindowDescriptor {
            title: "Rust Invaders!".to_string(),
            width: 598.0,
            height: 676.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup)
        .add_system(close_game)
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
        player: load_image(&mut images, PLAYER_SPRITE),
        laser: load_image(&mut images, LASER_SPRITE)
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
    })
        .insert(Player)
        .insert(Speed::default());
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Speed, &mut Transform), With<Player>>,
    win_size: Res<WinSize>
) {
    let left_boundary = -win_size.w / 2.;
    let right_boundary = win_size.w / 2.;
    if let Ok((speed, mut transform)) = query.get_single_mut() {
        let direction = if keyboard_input.pressed(KeyCode::A) { -1.}
        else if keyboard_input.pressed(KeyCode::D)  { 1.}
        else { 0.};
        transform.translation.x += direction * speed.0 * TIME_STEP;
    }
}

fn close_game(
    keyboard_input: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>
) {
    if keyboard_input.pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

fn player_fire(
    mut commands: Commands,
    kb: Res<Input<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut query: Query<&Transform , With<Player>>
){
    if let Ok(player_tf) = query.get_single_mut() {
        if kb.pressed(KeyCode::Space) {
            let coords = (player_tf.translation.x, player_tf.translation.y);
            let image = asset_server.load(LASER_SPRITE);
            commands.spawn_bundle(SpriteBundle {
                texture: image,
                transform: Transform {
                    translation: Vec3::new(coords.0, coords.1, 0.),
                    scale: Vec3::new(0.5, 0.5, 0.1),
                    ..Default::default()
                },
                ..Default::default()
            })
                .insert(Laser)
                .insert(Speed::default());
        }
    }
}

fn laser_movement(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Speed, &mut Transform), With<Laser>>
) {
    for (laser_entity, speed, mut laser_tf) in query.iter_mut() {
        let translation = &mut laser_tf.translation;
        translation.y += speed.0 * TIME_STEP;
        // if translation.y > win_size.h {
        //     commands.entity(laser_entity).despawn();
        // }
    }
}