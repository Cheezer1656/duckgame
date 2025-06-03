//! Renders a 2D scene containing a single, moving sprite.

use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const PLAYER_SPEED: f32 = 50.0;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Resource)]
struct Score(u32);

#[derive(Resource)]
struct BulletAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

#[derive(Component)]
struct IsPlayer;

#[derive(Component)]
struct IsEnemy;

#[derive(Component, Default)]
struct Velocity(Vec2);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "The Duck Game".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.0, 0.722, 0.961)))
        .insert_resource(Score(0))
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_input,
                update,
                spawn_enemies.run_if(on_timer(Duration::from_secs_f32(0.25))),
                spawn_bullets,
                check_for_collisions,
                check_for_player_collisions,
                update_score_text,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            OnEnter(GameState::GameOver),
            (darken_screen, display_game_over_text),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_image(asset_server.load("duck.png")),
        Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.3)),
        IsPlayer,
        Velocity::default(),
    ));

    let bullet_mesh = meshes.add(Rectangle::new(17.0, 6.0));
    let bullet_material = materials.add(ColorMaterial::from(Color::srgb(0.1, 0.1, 0.1)));
    commands.insert_resource(BulletAssets {
        mesh: bullet_mesh,
        material: bullet_material,
    });

    commands.spawn((
        Text2d::new("Score: 0"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        Transform::from_xyz(0.0, WINDOW_HEIGHT / 2.0 - 30.0, 0.0),
    ));
}

fn handle_input(
    mut query: Query<&mut Velocity, With<IsPlayer>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut vel) = query.single_mut() {
        if keyboard_input.pressed(KeyCode::KeyA) {
            vel.0.x -= PLAYER_SPEED;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            vel.0.x += PLAYER_SPEED;
        }
        if keyboard_input.pressed(KeyCode::KeyW) {
            vel.0.y += PLAYER_SPEED;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            vel.0.y -= PLAYER_SPEED;
        }
    }
}

fn update(mut query: Query<(&mut Transform, &mut Velocity, Option<&IsPlayer>)>, time: Res<Time>) {
    for (mut transform, mut vel, is_player) in query.iter_mut() {
        transform.translation.x += vel.0.x * time.delta_secs();
        transform.translation.y += vel.0.y * time.delta_secs();

        if is_player.is_some() {
            vel.0 *= 0.8; // Slow down the player over time
        }
    }
}

fn spawn_enemies(mut commands: Commands, asset_server: Res<AssetServer>) {
    if fastrand::u8(0..3) == 0 {
        commands.spawn((
            Sprite::from_image(asset_server.load("fish.png")),
            Transform::from_xyz(
                (WINDOW_WIDTH * 0.9 + fastrand::f32() * (WINDOW_WIDTH - WINDOW_WIDTH * 0.9)) / 2.0,
                -WINDOW_HEIGHT / 2.0 + fastrand::f32() * WINDOW_HEIGHT,
                0.0,
            )
            .with_scale(Vec3::splat(0.1)),
            Velocity(Vec2::new(-10.0 - fastrand::f32() * 30.0, 0.0)),
            IsEnemy,
        ));
    }
}

fn spawn_bullets(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<IsPlayer>>,
    bullet_assets: Res<BulletAssets>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(player_transform) = player_query.single() {
            commands.spawn((
                Mesh2d(bullet_assets.mesh.clone()),
                MeshMaterial2d(bullet_assets.material.clone()),
                Transform::from_xyz(
                    player_transform.translation.x + 70.0,
                    player_transform.translation.y + 14.0,
                    0.0,
                ),
                Velocity(Vec2::new(500.0, 0.0)),
            ));
        }
    }
}

fn check_for_collisions(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut bullet_query: Query<(Entity, &Transform), With<Mesh2d>>,
    enemy_query: Query<(Entity, &Transform), With<IsEnemy>>,
) {
    for (bullet_entity, bullet_transform) in bullet_query.iter_mut() {
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            if bullet_transform
                .translation
                .distance(enemy_transform.translation)
                < 30.0
            {
                commands.entity(bullet_entity).despawn();
                commands.entity(enemy_entity).despawn();
                score.0 += 1;
            }
        }
    }
}

fn check_for_player_collisions(
    player_query: Query<&Transform, With<IsPlayer>>,
    enemy_query: Query<&Transform, With<IsEnemy>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if let Ok(player_transform) = player_query.single() {
        for enemy_transform in enemy_query.iter() {
            if player_transform
                .translation
                .distance(enemy_transform.translation)
                < 50.0
            {
                game_state.set(GameState::GameOver);
            }
        }
    }
}

fn darken_screen(mut color: ResMut<ClearColor>) {
    color.0 = Color::srgb(0.1, 0.1, 0.1);
}

fn display_game_over_text(mut commands: Commands) {
    commands.spawn((
        Text2d::new("Game Over!"),
        TextFont {
            font_size: 50.0,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn update_score_text(score: Res<Score>, mut query: Query<&mut Text2d>) {
    if let Ok(mut text) = query.single_mut() {
        text.0 = format!("Score: {}", score.0);
    }
}
