use bevy::prelude::*;
use crate::{Player, PlayerState};

pub struct ArcherPlugin;

impl Plugin for ArcherPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_archer)
            .add_systems(Update, (
                spawn_wave3_archers,
                archer_ai,
                animate_archer,
                check_archer_shoot,
                check_arrow_collision,
                update_arrow,
                check_archer_death,
                player_attack_archer_collision,
                restrict_player_movement,
                play_archer_damage_sound, 
            ));
    }
}

#[derive(Component)]
struct Archer {
    hp: i32,
    direction_facing_right: bool,
    current_state: ArcherState,
    shoot_cooldown: Timer,
    has_shot: bool,
    was_hit_by_player: bool,
}

#[derive(Component)]
struct PreviousEnemyHp(i32);

#[derive(PartialEq, Clone, Copy)]
enum ArcherState {
    Idle,
    Walk,
    Shoot,
    Hurt,
    Dead,
}

#[derive(Component)]
struct Arrow {
    speed: f32,
    direction_facing_right: bool,
}

#[derive(Resource)]
struct ArcherAssets {
    idle_layout: Handle<TextureAtlasLayout>,
    walk_layout: Handle<TextureAtlasLayout>,
    shoot_layout: Handle<TextureAtlasLayout>,
    death_layout: Handle<TextureAtlasLayout>,
    hurt_layout: Handle<TextureAtlasLayout>,
    idle_texture: Handle<Image>,
    walk_texture: Handle<Image>,
    shoot_texture: Handle<Image>,
    death_texture: Handle<Image>,
    hurt_texture: Handle<Image>,
    arrow_texture: Handle<Image>,
}

#[derive(Component)]
struct AnimationConfig {
    timer: Timer,
    frame_count: usize,
}

fn setup_archer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut stage_progress: ResMut<crate::background::StageProgress>,
) {
    let idle_texture = asset_server.load("Skeleton_Archer/Idle.png");
    let idle_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 7, 1, None, None));
    let walk_texture = asset_server.load("Skeleton_Archer/Walk.png");
    let walk_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 8, 1, None, None));
    let shoot_texture = asset_server.load("Skeleton_Archer/Shoot1.png");
    let shoot_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 15, 1, None, None));
    let death_texture = asset_server.load("Skeleton_Archer/Dead.png");
    let death_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 5, 1, None, None));
    let hurt_texture = asset_server.load("Skeleton_Archer/Hurt.png");
    let hurt_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 2, 1, None, None));
    let arrow_texture = asset_server.load("Arrow.png");

    commands.spawn((
        Sprite {
            image: idle_texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: idle_layout.clone(),
                index: 0,
            }),
            custom_size: Some(Vec2::new(128.0, 128.0)),
            ..default()
        },
        Transform::from_xyz(350.0, -180.0, 4.0),
        Archer {
            hp: 40,
            direction_facing_right: true,
            current_state: ArcherState::Idle,
            shoot_cooldown: Timer::from_seconds(3.0, TimerMode::Repeating),
            has_shot: false,
            was_hit_by_player: false,
        },
        PreviousEnemyHp(40), 
        AnimationConfig {
            timer: Timer::from_seconds(0.12, TimerMode::Repeating),
            frame_count: 7,
        },
    ));
    stage_progress.archers_spawned = 1;

    commands.insert_resource(ArcherAssets {
        idle_layout, walk_layout, shoot_layout, death_layout, hurt_layout,
        idle_texture, walk_texture, shoot_texture, death_texture, hurt_texture, arrow_texture,
    });
}

fn spawn_wave3_archers(
    mut commands: Commands,
    archer_assets: Option<Res<ArcherAssets>>,
    mut stage_progress: ResMut<crate::background::StageProgress>,
) {
    let Some(assets) = archer_assets else { return; };

    if stage_progress.bg33_reached && !stage_progress.wave_3_spawned {
        let wave3_positions = [1500.0, 1650.0];
        for &x_pos in wave3_positions.iter() {
            commands.spawn((
                Sprite {
                    image: assets.idle_texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: assets.idle_layout.clone(),
                        index: 0,
                    }),
                    custom_size: Some(Vec2::new(128.0, 128.0)),
                    ..default()
                },
                Transform::from_xyz(x_pos, -180.0, 4.0),
                Archer {
                    hp: 40,
                    direction_facing_right: true,
                    current_state: ArcherState::Idle,
                    shoot_cooldown: Timer::from_seconds(3.0, TimerMode::Repeating),
                    has_shot: false,
                    was_hit_by_player: false,
                },
                PreviousEnemyHp(40), 
                AnimationConfig {
                    timer: Timer::from_seconds(0.12, TimerMode::Repeating),
                    frame_count: 7,
                },
            ));
            stage_progress.archers_spawned += 1;
        }
    }
}

fn archer_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut archer_query: Query<(&mut Archer, &mut Transform, &mut Sprite, &mut AnimationConfig), Without<Player>>,
    archer_assets: Res<ArcherAssets>,
) {
    // 👈 استفاده از متد فوق‌العاده امن iter().next() سازگار با تمامی نسخه‌های Bevy
    let Some(player_transform) = player_query.iter().next() else { return; };
    
    for (mut archer, mut archer_transform, mut sprite, mut config) in archer_query.iter_mut() {
        if archer.current_state == ArcherState::Dead || archer.current_state == ArcherState::Hurt { continue; }

        let player_pos = player_transform.translation;
        let archer_pos = archer_transform.translation;
        
        let x_direction = player_pos.x - archer_pos.x;
        let x_distance = x_direction.abs();
        let y_direction = player_pos.y - archer_pos.y;
        let y_distance = y_direction.abs();

        if x_direction > 0.0 {
            archer.direction_facing_right = true;
            sprite.flip_x = false;
        } else {
            archer.direction_facing_right = false;
            sprite.flip_x = true;
        }

        let mut is_moving = false;
        if x_distance > 550.0 {
            archer_transform.translation.x += x_direction.signum() * 80.0 * time.delta_secs();
            is_moving = true;
        }

        if y_distance > 10.0 {
            archer_transform.translation.y += y_direction.signum() * 80.0 * time.delta_secs();
            is_moving = true;
        }

        if is_moving {
            if archer.current_state != ArcherState::Walk {
                archer.current_state = ArcherState::Walk;
                sprite.image = archer_assets.walk_texture.clone();
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.layout = archer_assets.walk_layout.clone();
                    atlas.index = 0;
                }
                config.frame_count = 8;
                config.timer.set_duration(std::time::Duration::from_secs_f32(0.1));
            }
        } else {
            if archer.current_state == ArcherState::Walk {
                archer.current_state = ArcherState::Idle;
            }
            
            archer.shoot_cooldown.tick(time.delta());
            if archer.shoot_cooldown.just_finished() {
                if archer.current_state != ArcherState::Shoot {
                    archer.current_state = ArcherState::Shoot;
                    archer.has_shot = false;
                    sprite.image = archer_assets.shoot_texture.clone();
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.layout = archer_assets.shoot_layout.clone();
                        atlas.index = 0;
                    }
                    config.frame_count = 15;
                    config.timer.set_duration(std::time::Duration::from_secs_f32(0.08));
                }
            } else if archer.current_state != ArcherState::Shoot {
                if archer.current_state != ArcherState::Idle {
                    archer.current_state = ArcherState::Idle;
                    sprite.image = archer_assets.idle_texture.clone();
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.layout = archer_assets.idle_layout.clone();
                        atlas.index = 0;
                    }
                    config.frame_count = 7;
                    config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
                }
            }
        }
    }
}

fn animate_archer(
    time: Res<Time>,
    mut query: Query<(&mut Archer, &mut Sprite, &mut AnimationConfig), With<Archer>>, // 👈 تگ With<Archer> برای جداسازی انیمیشن کماندار از جادوگر
    archer_assets: Res<ArcherAssets>,
) {
    for (mut archer, mut sprite, mut config) in query.iter_mut() {
        config.timer.tick(time.delta());
        if config.timer.just_finished() {
            let mut should_reset_to_idle = false;

            if let Some(atlas) = &mut sprite.texture_atlas {
                if archer.current_state == ArcherState::Shoot || archer.current_state == ArcherState::Dead {
                    if atlas.index < config.frame_count - 1 {
                        atlas.index += 1;
                    }
                } else if archer.current_state == ArcherState::Hurt {
                    if atlas.index < config.frame_count - 1 {
                        atlas.index += 1;
                    } else {
                        should_reset_to_idle = true;
                    }
                } else {
                    atlas.index = (atlas.index + 1) % config.frame_count;
                }
            }

            if should_reset_to_idle {
                archer.current_state = ArcherState::Idle;
                sprite.image = archer_assets.idle_texture.clone();
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.layout = archer_assets.idle_layout.clone();
                    atlas.index = 0;
                }
                config.frame_count = 7;
                config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
            }
        }
    }
}

fn check_archer_shoot(
    mut commands: Commands,
    archer_assets: Res<ArcherAssets>,
    mut archer_query: Query<(&mut Archer, &Transform, &Sprite)>,
) {
    for (mut archer, transform, sprite) in archer_query.iter_mut() {
        if archer.current_state == ArcherState::Shoot {
            if let Some(atlas) = &sprite.texture_atlas {
                if atlas.index == 9 && !archer.has_shot {
                    archer.has_shot = true;
                    
                    let arrow_x_offset = if archer.direction_facing_right { 30.0 } else { -30.0 };
                    let arrow_pos = transform.translation + Vec3::new(arrow_x_offset, 0.0, 0.0);
                    
                    commands.spawn((
                        Sprite {
                            image: archer_assets.arrow_texture.clone(),
                            custom_size: Some(Vec2::new(32.0, 32.0)),
                            flip_x: !archer.direction_facing_right,
                            ..default()
                        },
                        Transform::from_translation(arrow_pos),
                        Arrow {
                            speed: 100.0,
                            direction_facing_right: archer.direction_facing_right,
                        },
                    ));
                }
            }
        }
    }
}

fn check_arrow_collision(
    mut commands: Commands,
    arrow_query: Query<(Entity, &Transform, &Arrow)>,
    mut player_query: Query<(&Transform, &mut Player)>,
) {
    // 👈 استفاده ایمن از iter_mut().next() برای جلوگیری از تداخل فریمورک
    let Some((player_transform, mut player)) = player_query.iter_mut().next() else { return; };
    let player_pos = player_transform.translation;

    for (arrow_entity, arrow_transform, _arrow) in arrow_query.iter() {
        let arrow_pos = arrow_transform.translation;
        let y_distance = (player_pos.y - arrow_pos.y).abs();

        if y_distance < 20.0 {
            let dx = arrow_pos.x - player_pos.x;
            let dy = arrow_pos.y - player_pos.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < 30.0 {
                commands.entity(arrow_entity).despawn();
                if player.hp > 0 {
                    player.hp -= 20;
                }
                println!("تیر به بازیکن خورد: {}", player.hp);
            }
        }
    }
}

fn update_arrow(time: Res<Time>, mut arrow_query: Query<(&mut Transform, &Arrow)>) {
    for (mut arrow_transform, arrow) in arrow_query.iter_mut() {
        let direction = if arrow.speed > 0.0 {
            if arrow.direction_facing_right { 1.0 } else { -1.0 }
        } else { 0.0 };
        arrow_transform.translation.x += direction * arrow.speed * time.delta_secs();
    }
}

fn player_attack_archer_collision(
    player_query: Query<(&Transform, &Player)>,
    mut archer_query: Query<(&mut Archer, &mut Sprite, &mut AnimationConfig, &Transform)>,
    archer_assets: Res<ArcherAssets>,
) {
    // 👈 استفاده ایمن از iter().next()
    let Some((player_transform, player)) = player_query.iter().next() else { return; };
    
    if player.current_state == PlayerState::Attacking {
        for (mut archer, mut sprite, mut config, archer_transform) in archer_query.iter_mut() {
            if archer.current_state == ArcherState::Dead { continue; }

            let player_pos = player_transform.translation;
            let archer_pos = archer_transform.translation;

            let x_distance = (player_pos.x - archer_pos.x).abs();
            let y_distance = (player_pos.y - archer_pos.y).abs();

            if x_distance < 60.0 && y_distance < 20.0 {
                if !archer.was_hit_by_player {
                    archer.hp -= 20;
                    archer.was_hit_by_player = true;
                    println!("شما به آرچر ضربه زدید: {}", archer.hp);

                    if archer.hp > 0 {
                        archer.current_state = ArcherState::Hurt;
                        sprite.image = archer_assets.hurt_texture.clone();
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            atlas.layout = archer_assets.hurt_layout.clone();
                            atlas.index = 0;
                        }
                        config.frame_count = 2;
                        config.timer.set_duration(std::time::Duration::from_secs_f32(0.15));
                    }
                }
            }
        }
    } else {
        for (mut archer, _, _, _) in archer_query.iter_mut() {
            archer.was_hit_by_player = false;
        }
    }
}

fn check_archer_death(
    anim_assets: Res<ArcherAssets>,
    mut query: Query<(Entity, &mut Archer, &mut Sprite, &mut AnimationConfig)>,
    mut stage_progress: ResMut<crate::background::StageProgress>,
    mut commands: Commands,
) {
    for (entity, mut archer, mut sprite, mut config) in query.iter_mut() {
        if archer.hp <= 0 && archer.current_state != ArcherState::Dead {
            archer.current_state = ArcherState::Dead;
            println!("یک آرچر مرد!");
            
            sprite.image = anim_assets.death_texture.clone();
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.layout = anim_assets.death_layout.clone();
                atlas.index = 0;
            }
            config.frame_count = 5;
            config.timer.set_duration(std::time::Duration::from_secs_f32(0.15));
        }

        if archer.current_state == ArcherState::Dead {
            if let Some(atlas) = &sprite.texture_atlas {
                if atlas.index == config.frame_count - 1 {
                    stage_progress.archers_killed += 1;
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn restrict_player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    stage_progress: Res<crate::background::StageProgress>,
) {
    if stage_progress.archers_killed < 1 {
        // 👈 استفاده ایمن از iter_mut().next() برای محدودیت حرکت بازیکن
        if let Some(mut player_transform) = player_query.iter_mut().next() {
            let max_player_x = 800.0; 
            if player_transform.translation.x > max_player_x {
                player_transform.translation.x = max_player_x;
            }
        }
    }
}

fn play_archer_damage_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut query: Query<(&Archer, &mut PreviousEnemyHp)>,
) {
    for (archer, mut prev_hp) in query.iter_mut() {
        if archer.hp < prev_hp.0 {
            if archer.hp <= 0 {
                commands.spawn(AudioPlayer::new(asset_server.load("md2.mp3")));
            } else {
                commands.spawn(AudioPlayer::new(asset_server.load("md3.mp3")));
            }
        }
        prev_hp.0 = archer.hp;
    }
}