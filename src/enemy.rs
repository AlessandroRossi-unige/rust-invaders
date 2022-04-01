use bevy::core::FixedTimestep;
use bevy::prelude::*;
use crate::{ActiveEnemies, App, Enemy, ENEMY_SPRITE, FromEnemy, FromPlayer, Laser, SCALE, Speed, SpriteInfos, TIME_STEP, WinSize};
use rand::{Rng, thread_rng};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(enemy_laser_movement.system())
            .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(enemy_spawn.system())
        ).add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.9))
                .with_system(enemy_fire.system())
        );
    }
}

fn enemy_spawn(
    mut commands: Commands,
    mut active_enemies: ResMut<ActiveEnemies>,
    win_size: Res<WinSize>,
    textures: Res<SpriteInfos>
) {
    if active_enemies.0 < 1 {
        let mut rng = thread_rng();
        let w_spawn = win_size.w /  2. - 100.;
        let h_spawn = win_size.h /  2. - 100.;
        let x = rng.gen_range(-w_spawn..w_spawn) as f32;
        let y = rng.gen_range(-h_spawn..h_spawn) as f32;

        commands.spawn_bundle(SpriteBundle {
            texture: textures.enemy.0.clone(),
            transform: Transform {
                translation: Vec3::new(x, y, 10.),
                scale: Vec3::new(SCALE, SCALE, 0.5),
                ..Default::default()
            },
            ..Default::default()
        })
            .insert(Enemy);

        active_enemies.0 += 1;
    }
}

fn enemy_fire(
    mut commands: Commands,
    textures: Res<SpriteInfos>,
    mut enemy_query: Query<(&Transform), With<Enemy>>
){
    for &tf in enemy_query.iter() {
        let coords = (tf.translation.x, tf.translation.y);
        commands
            .spawn_bundle(SpriteBundle {
                texture: textures.enemy_laser.0.clone(),
                transform: Transform{
                    translation: Vec3::new(coords.0, coords.1 - 15.0, 0.),
                    scale: Vec3::new(SCALE, -SCALE, 1.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Laser)
            .insert(FromEnemy)
            .insert(Speed::default());
    }
}

fn enemy_laser_movement(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut laser_query: Query<(Entity, &Speed, &mut Transform), (With<Laser>, With<FromEnemy>)>,
) {
    // for each laser from enemy
    for (entity, speed, mut tf) in laser_query.iter_mut() {
        tf.translation.y -= speed.0 * TIME_STEP;
        if tf.translation.y < -win_size.h / 2. - 50. {
            commands.entity(entity).despawn();
        }
    }
}