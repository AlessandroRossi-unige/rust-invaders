#![allow(unused)]


mod player;
mod enemy;

use bevy::math::Vec3Swizzles;
use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::path::Path;
use std::process::exit;
use std::ptr::addr_of;
use bevy::app::AppExit;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::texture::ImageType;
use bevy::sprite::collide_aabb::collide;
use bevy::window::CloseWindow;
use crate::enemy::EnemyPlugin;
use crate::player::PlayerPlugin;
use crate::StartupStage::Startup;

const SPRITE_DIR: &str = "assets";

const PLAYER_SPRITE: &str = "player_a_01.png";
const ENEMY_SPRITE: &str = "enemy_a_01.png";
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const SCALE: f32 = 0.5;
const PLAYER_RESPAWN_DELAY: f64 = 2.;

const TIME_STEP: f32 = 1. / 60.;

pub struct SpriteInfos {
    player: (Handle<Image>, Vec2),
    player_laser: (Handle<Image>, Vec2),
    enemy_laser: (Handle<Image>, Vec2),
    enemy: (Handle<Image>, Vec2),
    explosion: Handle<TextureAtlas>

}

struct WinSize {
    w: f32,
    h: f32,
}

struct PlayerState {
    is_alive: bool,
    last_shot: f64
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            is_alive: false,
            last_shot: 0.
        }
    }
}

impl PlayerState {
    fn shot(&mut self, time: f64) {
        self.is_alive = false;
        self.last_shot = time;
    }
    fn spawned(&mut self) {
        self.is_alive = true;
        self.last_shot = 0.;
    }
}

#[derive(Component)]
struct ActiveEnemies(u32);
#[derive(Component)]
struct Enemy;
#[derive(Component)]
struct Player;
#[derive(Component)]
struct Laser;
#[derive(Component)]
struct PlayerReadyFire(bool);
#[derive(Component)]
struct Explosion;
#[derive(Component)]
struct ExplosionToSpawn(Vec3);
#[derive(Component)]
struct FromPlayer;
#[derive(Component)]
struct FromEnemy;

#[derive(Component)]
struct Speed(f32);

impl Default for Speed {
    fn default() -> Self {
        Self(500.)
    }
}

fn main() {
    App::new()
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(WindowDescriptor {
            title: "Rust Invaders!".to_string(),
            width: 598.0,
            height: 676.0,
            ..Default::default()
        })
        .insert_resource(ActiveEnemies(0))
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin)
        .add_startup_system(setup)
        .add_system(close_game)
        .add_system(player_laser_hit_enemy.system())
        .add_system(enemy_laser_hit_player.system())
        .add_system(explosion_to_spawn.system())
        .add_system(animate_explosion.system())
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

    // create the main resources
    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 4);

    commands.insert_resource(SpriteInfos {
        player: load_image(&mut images, PLAYER_SPRITE),
        player_laser: load_image(&mut images, PLAYER_LASER_SPRITE),
        enemy_laser: load_image(&mut images, ENEMY_LASER_SPRITE),
        enemy: load_image(&mut images, ENEMY_SPRITE),
        explosion: texture_atlases.add(texture_atlas),
    });

}

fn close_game(
    keyboard_input: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>
) {
    if keyboard_input.pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
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


fn player_laser_hit_enemy(
    mut commands: Commands,
    sprite_infos: Res<SpriteInfos>,
    mut laser_query: Query<(Entity, &Transform), (With<Laser>, With<FromPlayer>)>,
    mut enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut active_enemies: ResMut<ActiveEnemies>,
) {

    let mut enemies_blasted: HashSet<Entity> = HashSet::new();

    for (player_laser_entity, player_laser_tf) in laser_query.iter_mut() {
        let player_laser_size = sprite_infos.player_laser.1;
        let player_laser_scale = Vec2::from(player_laser_tf.scale.abs().xy());
        for (enemy_entity, enemy_tf) in enemy_query.iter_mut() {

            let enemy_size = sprite_infos.enemy.1;
            let enemy_scale = Vec2::from(enemy_tf.scale.xy());

            let collision = collide(
                player_laser_tf.translation,
                player_laser_size * player_laser_scale,
                enemy_tf.translation,
                enemy_size * enemy_scale,
            );

            if let Some(_) = collision {
                if enemies_blasted.get(&enemy_entity).is_none() {
                    // remove the enemy
                    commands.entity(enemy_entity).despawn();
                    active_enemies.0 -= 1;

                    commands
                        .spawn()
                        .insert(ExplosionToSpawn(enemy_tf.translation.clone()));

                    enemies_blasted.insert(enemy_entity);
                }
                // remove the laser
                commands.entity(player_laser_entity).despawn();
            }
        }
    }
}

fn enemy_laser_hit_player(
    mut commands: Commands,
    sprite_infos: Res<SpriteInfos>,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    laser_query: Query<(Entity, &Transform), (With<Laser>, With<FromEnemy>)>,
    player_query: Query<(Entity, &Transform), With<Player>>,
) {
    if let Ok((player_entity, player_tf)) = player_query.get_single() {
        let player_size = sprite_infos.player.1;
        let player_scale = Vec2::from(player_tf.scale.xy());

        // for each enemy laser
        for (enemy_laser_entity, enemy_laser_tf) in laser_query.iter() {
            let enemy_laser_scale = Vec2::from(enemy_laser_tf.scale.abs().xy());
            let enemy_laser_size = sprite_infos.enemy_laser.1;

            let collision = collide(
                enemy_laser_tf.translation,
                enemy_laser_size * enemy_laser_scale,
                player_tf.translation,
                player_size * player_scale,
            );

            // process collision
            if let Some(_) = collision {
                // remove the player
                commands.entity(player_entity).despawn();
                player_state.shot(time.seconds_since_startup());
                // remove the laser
                commands.entity(enemy_laser_entity).despawn();
                // spawn the ExplosionToSpawn entity
                commands
                    .spawn()
                    .insert(ExplosionToSpawn(player_tf.translation.clone()));
            }
        }
    }
}

fn explosion_to_spawn(
    mut commands: Commands,
    query: Query<(Entity, &ExplosionToSpawn)>,
    materials: Res<SpriteInfos>,
) {
    for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: materials.explosion.clone(),
                transform: Transform {
                    translation: explosion_to_spawn.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Explosion)
            .insert(Timer::from_seconds(0.05, true));

        commands.entity(explosion_spawn_entity).despawn();
    }
}

fn animate_explosion(
    mut commands: Commands,
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<
        (
            Entity,
            &mut Timer,
            &mut TextureAtlasSprite,
            &Handle<TextureAtlas>,
        ),
        With<Explosion>,
    >,
) {
    for (entity, mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index += 1;
            if sprite.index == texture_atlas.textures.len() {
                commands.entity(entity).despawn();
            }
        }
    }
}