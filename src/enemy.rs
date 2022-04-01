use bevy::core::FixedTimestep;
use bevy::prelude::*;
use crate::{ActiveEnemies, App, Enemy, ENEMY_SPRITE, SCALE, WinSize};
use rand::{Rng, thread_rng};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(enemy_spawn.system())
        );
    }
}

fn enemy_spawn(
    mut commands: Commands,
    mut active_enemies: ResMut<ActiveEnemies>,
    win_size: Res<WinSize>,
    asset_server: Res<AssetServer>
) {
    if active_enemies.0 < 1 {
        let mut rng = thread_rng();
        let w_spawn = win_size.w /  2. - 100.;
        let h_spawn = win_size.h /  2. - 100.;
        let x = rng.gen_range(-w_spawn..w_spawn) as f32;
        let y = rng.gen_range(-h_spawn..h_spawn) as f32;

        let image = asset_server.load(ENEMY_SPRITE);
        commands.spawn_bundle(SpriteBundle {
            texture: image,
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