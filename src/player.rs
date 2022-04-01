use bevy::core::FixedTimestep;
use bevy::prelude::*;
use crate::{FromPlayer, Laser, Player, PLAYER_LASER_SPRITE, PLAYER_RESPAWN_DELAY, PLAYER_SPRITE, PlayerReadyFire, PlayerState, SCALE, Speed, TIME_STEP, WinSize};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PlayerState::default())
            .add_startup_stage(
                "game_setup_actors",
                SystemStage::single(player_spawn.system()),
            )
            .add_system(player_movement.system())
            .add_system(player_fire.system())
            .add_system(laser_movement.system())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.5))
                    .with_system(player_spawn.system())
            );

    }
}

fn player_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    win_size: Res<WinSize>,
    time: Res<Time>,
    mut player_state: ResMut<PlayerState>
) {
    let now = time.seconds_since_startup();
    let last_shot = player_state.last_shot;
    let bottom = win_size.h / 2.;

    if !player_state.is_alive && (last_shot == 0. || now > last_shot + PLAYER_RESPAWN_DELAY) {
        let image = asset_server.load(PLAYER_SPRITE);
        commands.spawn_bundle(SpriteBundle {
            texture: image,
            transform: Transform {
                translation: Vec3::new(0., -bottom + 25., 10.),
                scale: Vec3::new(SCALE, SCALE, 0.1),
                ..Default::default()
            },
            ..Default::default()
        })
            .insert(Player)
            .insert(Speed::default())
            .insert(PlayerReadyFire(true));

        player_state.spawned();
    }
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Speed, &mut Transform), With<Player>>,
    win_size: Res<WinSize>
) {
    let x_offset = 100. / 2.;
    let boundary = win_size.w / 2.;
    if let Ok((speed, mut transform)) = query.get_single_mut() {
        let direction = if keyboard_input.pressed(KeyCode::A) && (transform.translation.x - x_offset) > -boundary { -1.}
        else if keyboard_input.pressed(KeyCode::D)  && (transform.translation.x + x_offset) < boundary  { 1.}
        else { 0.};
        transform.translation.x += direction * speed.0 * TIME_STEP;
    }
}



fn player_fire(
    mut commands: Commands,
    kb: Res<Input<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&Transform, &mut PlayerReadyFire), With<Player>>
){
    if let Ok((player_tf, mut ready_fire)) = query.get_single_mut() {
        if ready_fire.0 && kb.pressed(KeyCode::Space) {
            let coords = (player_tf.translation.x, player_tf.translation.y);

            let mut spawn_laser = |x_offset: f32| {
                let image = asset_server.load(PLAYER_LASER_SPRITE);
                commands.spawn_bundle(SpriteBundle {
                    texture: image,
                    transform: Transform {
                        translation: Vec3::new(coords.0 + x_offset, coords.1 + 15., 0.),
                        scale: Vec3::new(0.4, 0.4, 0.1),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                    .insert(Laser)
                    .insert(FromPlayer)
                    .insert(Speed::default());
            };
            let x_offset = 144. / 4. - 5.;
            spawn_laser(x_offset);
            spawn_laser(-x_offset);

            ready_fire.0 = false;
        }

        if kb.just_released(KeyCode::Space) {
            ready_fire.0 = true;
        }
    }
}

fn laser_movement(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Speed, &mut Transform), (With<Laser>, With<FromPlayer>)>
) {
    for (laser_entity, speed, mut laser_tf) in query.iter_mut() {
        let translation = &mut laser_tf.translation;
        translation.y += speed.0 * TIME_STEP;
        if translation.y > win_size.h {
            commands.entity(laser_entity).despawn();
        }
    }
}