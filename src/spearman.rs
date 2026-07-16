use bevy::prelude::*;
use crate::{Player, PlayerState};

pub struct SpearmanPlugin;

impl Plugin for SpearmanPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                spawn_spearman_trigger,
                spawn_wave3_spearmen,
                spearman_ai,
                animate_spearman,
                check_spearman_death,
                player_attack_spearman_collision,
                restrict_player_spearman_movement,
                play_spearman_damage_sound, 
            ));
    }
}

#[derive(Component)]
struct Spearman {
    hp: i32,
    direction_facing_right: bool,
    current_state: SpearmanState,
    attack_cooldown: Timer,
    was_hit_by_player: bool,
    is_running_charge: bool,
    combo_index: u8,
}

#[derive(Component)]
struct PreviousEnemyHp(i32);

#[derive(PartialEq, Clone, Copy)]
enum SpearmanState {
    Idle,
    Walk,
    Run,
    Attack1,
    Attack2,
    AttackRun,
    Dead,
}

#[derive(Resource)]
struct SpearmanAssets {
    idle_layout: Handle<TextureAtlasLayout>,
    walk_layout: Handle<TextureAtlasLayout>,
    run_layout: Handle<TextureAtlasLayout>,
    attack1_layout: Handle<TextureAtlasLayout>,
    attack2_layout: Handle<TextureAtlasLayout>,
    attack_run_layout: Handle<TextureAtlasLayout>,
    death_layout: Handle<TextureAtlasLayout>,
    idle_texture: Handle<Image>,
    walk_texture: Handle<Image>,
    run_texture: Handle<Image>,
    attack1_texture: Handle<Image>,
    attack2_texture: Handle<Image>,
    attack_run_texture: Handle<Image>,
    death_texture: Handle<Image>,
}

#[derive(Component)]
struct AnimationConfig {
    timer: Timer,
    frame_count: usize,
}

fn spawn_spearman_trigger(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    player_query: Query<&Transform, With<Player>>,
    mut stage_progress: ResMut<crate::background::StageProgress>,
) {
    if stage_progress.archers_killed >= 1 && stage_progress.spearmen_spawned == 0 {
        if let Ok(player_transform) = player_query.single() {
            if player_transform.translation.x > 900.0 {
                
                let idle_texture = asset_server.load("Skeleton_Spearman/Idle.png");
                let idle_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 7, 1, None, None));
                let walk_texture = asset_server.load("Skeleton_Spearman/Walk.png");
                let walk_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 7, 1, None, None));
                let run_texture = asset_server.load("Skeleton_Spearman/Run.png");
                let run_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 6, 1, None, None));
                let attack1_texture = asset_server.load("Skeleton_Spearman/Attack_1.png");
                let attack1_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 4, 1, None, None));
                let attack2_texture = asset_server.load("Skeleton_Spearman/Attack_2.png");
                let attack2_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 4, 1, None, None));
                let attack_run_texture = asset_server.load("Skeleton_Spearman/Run+attack.png");
                let attack_run_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 5, 1, None, None));
                let death_texture = asset_server.load("Skeleton_Spearman/Dead.png");
                let death_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 5, 1, None, None));

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
                    Transform::from_xyz(1800.0, -180.0, 4.0),
                    Spearman {
                        hp: 60,
                        direction_facing_right: false,
                        current_state: SpearmanState::Idle,
                        attack_cooldown: Timer::from_seconds(1.5, TimerMode::Repeating),
                        was_hit_by_player: false,
                        is_running_charge: false,
                        combo_index: 1,
                    },
                    PreviousEnemyHp(60), 
                    AnimationConfig {
                        timer: Timer::from_seconds(0.12, TimerMode::Repeating),
                        frame_count: 7,
                    },
                ));
                stage_progress.spearmen_spawned = 1;

                commands.insert_resource(SpearmanAssets {
                    idle_layout, walk_layout, run_layout, attack1_layout, attack2_layout, attack_run_layout, death_layout,
                    idle_texture, walk_texture, run_texture, attack1_texture, attack2_texture, attack_run_texture, death_texture,
                });

                println!("۱ نیزه‌دار در بک‌گراند ۲ وارد میدان شد!");
            }
        }
    }
}

fn spawn_wave3_spearmen(
    mut commands: Commands,
    spearman_assets: Option<Res<SpearmanAssets>>,
    mut stage_progress: ResMut<crate::background::StageProgress>,
) {
    let Some(assets) = spearman_assets else { return; };

    if stage_progress.bg33_reached && !stage_progress.wave_3_spawned {
        let wave3_spear_positions = [1350.0, 1450.0, 1550.0];
        for &x_pos in wave3_spear_positions.iter() {
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
                Spearman {
                    hp: 60,
                    direction_facing_right: true,
                    current_state: SpearmanState::Idle,
                    attack_cooldown: Timer::from_seconds(1.5, TimerMode::Repeating),
                    was_hit_by_player: false,
                    is_running_charge: false,
                    combo_index: 1,
                },
                PreviousEnemyHp(60), 
                AnimationConfig {
                    timer: Timer::from_seconds(0.12, TimerMode::Repeating),
                    frame_count: 7,
                },
            ));
            stage_progress.spearmen_spawned += 1;
        }
        stage_progress.wave_3_spawned = true;
        println!("-> ۳ نیزه‌دار موج نهایی یکهو از عقب (بک‌گراند ۲) حمله کردند!");
    }
}

fn spearman_ai(
    time: Res<Time>,
    mut player_query: Query<(&Transform, &mut Player)>,
    mut spearman_query: Query<(&mut Spearman, &mut Transform, &mut Sprite, &mut AnimationConfig), Without<Player>>,
    spearman_assets: Option<Res<SpearmanAssets>>,
) {
    let Some(assets) = spearman_assets else { return; };

    if let Ok((player_transform, mut player)) = player_query.single_mut() {
        for (mut spearman, mut spearman_transform, mut sprite, mut config) in spearman_query.iter_mut() {
            if spearman.current_state == SpearmanState::Dead { continue; }

            if matches!(spearman.current_state, SpearmanState::Attack1 | SpearmanState::Attack2 | SpearmanState::AttackRun) {
                continue;
            }

            let player_pos = player_transform.translation;
            let spearman_pos = spearman_transform.translation;
            
            let x_direction = player_pos.x - spearman_pos.x;
            let x_distance = x_direction.abs();
            let y_direction = player_pos.y - spearman_pos.y;
            let y_distance = y_direction.abs();

            if x_direction > 0.0 {
                spearman.direction_facing_right = true;
                sprite.flip_x = false;
            } else {
                spearman.direction_facing_right = false;
                sprite.flip_x = true;
            }

            if y_distance > 10.0 {
                spearman_transform.translation.y += y_direction.signum() * 90.0 * time.delta_secs();
            }

            if spearman.is_running_charge {
                if x_distance < 70.0 {
                    spearman.current_state = SpearmanState::AttackRun;
                    spearman.is_running_charge = false; 
                    sprite.image = assets.attack_run_texture.clone();
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.layout = assets.attack_run_layout.clone();
                        atlas.index = 0;
                    }
                    config.frame_count = 5;
                    config.timer.set_duration(std::time::Duration::from_secs_f32(0.08));
                    
                    if y_distance < 20.0 {
                        if player.hp > 0 { player.hp -= 30; }
                    }
                } else {
                    spearman_transform.translation.x += x_direction.signum() * 140.0 * time.delta_secs();
                    if spearman.current_state != SpearmanState::Run {
                        spearman.current_state = SpearmanState::Run;
                        sprite.image = assets.run_texture.clone();
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            atlas.layout = assets.run_layout.clone();
                            atlas.index = 0;
                        }
                        config.frame_count = 6;
                        config.timer.set_duration(std::time::Duration::from_secs_f32(0.1));
                    }
                }
            } else {
                if x_distance < 50.0 {
                    spearman.attack_cooldown.tick(time.delta());
                    if spearman.attack_cooldown.just_finished() {
                        spearman.current_state = SpearmanState::Attack1;
                        spearman.combo_index = 1;
                        sprite.image = assets.attack1_texture.clone();
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            atlas.layout = assets.attack1_layout.clone();
                            atlas.index = 0;
                        }
                        config.frame_count = 4;
                        config.timer.set_duration(std::time::Duration::from_secs_f32(0.1));
                    } else if spearman.current_state != SpearmanState::Idle {
                        spearman.current_state = SpearmanState::Idle;
                        sprite.image = assets.idle_texture.clone();
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            atlas.layout = assets.idle_layout.clone();
                            atlas.index = 0;
                        }
                        config.frame_count = 7;
                        config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
                    }
                } else if x_distance >= 50.0 && x_distance <= 220.0 {
                    spearman_transform.translation.x += x_direction.signum() * 70.0 * time.delta_secs();
                    if spearman.current_state != SpearmanState::Walk {
                        spearman.current_state = SpearmanState::Walk;
                        sprite.image = assets.walk_texture.clone();
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            atlas.layout = assets.walk_layout.clone();
                            atlas.index = 0;
                        }
                        config.frame_count = 7;
                        config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
                    }
                } else if x_distance > 220.0 {
                    spearman.is_running_charge = true;
                }
            }
        }
    }
}

fn animate_spearman(
    time: Res<Time>,
    mut player_query: Query<&mut Player>,
    mut query: Query<(&mut Spearman, &mut Sprite, &mut AnimationConfig)>,
    spearman_assets: Option<Res<SpearmanAssets>>,
) {
    let Some(assets) = spearman_assets else { return; };

    if let Ok(mut player_res) = player_query.single_mut() {
        for (mut spearman, mut sprite, mut config) in query.iter_mut() {
            config.timer.tick(time.delta());
            if config.timer.just_finished() {
                let mut attack_finished = false;

                if let Some(atlas) = &mut sprite.texture_atlas {
                    if matches!(spearman.current_state, SpearmanState::Attack1 | SpearmanState::Attack2 | SpearmanState::AttackRun) {
                        if atlas.index < config.frame_count - 1 {
                            atlas.index += 1;
                        } else {
                            attack_finished = true;
                        }
                    } else if spearman.current_state == SpearmanState::Dead {
                        if atlas.index < config.frame_count - 1 {
                            atlas.index += 1;
                        }
                    } else {
                        atlas.index = (atlas.index + 1) % config.frame_count;
                    }
                }

                if attack_finished {
                    match spearman.current_state {
                        SpearmanState::Attack1 => {
                            if player_res.hp > 0 { player_res.hp -= 10; }
                            spearman.combo_index = 2;
                            spearman.current_state = SpearmanState::Attack2;
                            sprite.image = assets.attack2_texture.clone();
                            if let Some(atlas) = &mut sprite.texture_atlas {
                                atlas.layout = assets.attack2_layout.clone();
                                atlas.index = 0;
                            }
                            config.frame_count = 4;
                            config.timer.set_duration(std::time::Duration::from_secs_f32(0.1));
                        }
                        SpearmanState::Attack2 => {
                            if player_res.hp > 0 { player_res.hp -= 15; }
                            spearman.combo_index = 1;
                            spearman.current_state = SpearmanState::Idle;
                            sprite.image = assets.idle_texture.clone();
                            if let Some(atlas) = &mut sprite.texture_atlas {
                                atlas.layout = assets.idle_layout.clone();
                                atlas.index = 0;
                            }
                            config.frame_count = 7;
                            config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
                        }
                        _ => {
                            spearman.current_state = SpearmanState::Idle;
                            sprite.image = assets.idle_texture.clone();
                            if let Some(atlas) = &mut sprite.texture_atlas {
                                atlas.layout = assets.idle_layout.clone();
                                atlas.index = 0;
                            }
                            config.frame_count = 7;
                            config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
                        }
                    }
                }
            }
        }
    }
}

fn player_attack_spearman_collision(
    player_query: Query<(&Transform, &Player)>,
    mut spearman_query: Query<(&mut Spearman, &Transform)>,
) {
    if let Ok((player_transform, player)) = player_query.single() {
        if player.current_state == PlayerState::Attacking {
            for (mut spearman, spearman_transform) in spearman_query.iter_mut() {
                if spearman.current_state == SpearmanState::Dead { continue; }

                let player_pos = player_transform.translation;
                let spearman_pos = spearman_transform.translation;

                let x_distance = (player_pos.x - spearman_pos.x).abs();
                let y_distance = (player_pos.y - spearman_pos.y).abs();

                if x_distance < 60.0 && y_distance < 20.0 {
                    if !spearman.was_hit_by_player {
                        spearman.hp -= 20;
                        spearman.was_hit_by_player = true;
                        println!("به نیزه‌دار ضربه زدید! HP: {}", spearman.hp);
                    }
                }
            }
        } else {
            for (mut spearman, _) in spearman_query.iter_mut() {
                spearman.was_hit_by_player = false;
            }
        }
    }
}

fn check_spearman_death(
    spearman_assets: Option<Res<SpearmanAssets>>,
    mut query: Query<(Entity, &mut Spearman, &mut Sprite, &mut AnimationConfig)>,
    mut stage_progress: ResMut<crate::background::StageProgress>,
    mut commands: Commands,
) {
    let Some(assets) = spearman_assets else { return; };

    for (entity, mut spearman, mut sprite, mut config) in query.iter_mut() {
        if spearman.hp <= 0 && spearman.current_state != SpearmanState::Dead {
            spearman.current_state = SpearmanState::Dead;
            println!("یک نیزه‌دار مرد!");
            
            sprite.image = assets.death_texture.clone();
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.layout = assets.death_layout.clone();
                atlas.index = 0;
            }
            config.frame_count = 5;
            config.timer.set_duration(std::time::Duration::from_secs_f32(0.15));
        }

        if spearman.current_state == SpearmanState::Dead {
            if let Some(atlas) = &sprite.texture_atlas {
                if atlas.index == config.frame_count - 1 {
                    stage_progress.spearmen_killed += 1;
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn restrict_player_spearman_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    stage_progress: Res<crate::background::StageProgress>,
) {
    if let Ok(mut player_transform) = player_query.single_mut() {
        if stage_progress.spearmen_killed < 1 {
            let max_player_x = 2900.0;
            if player_transform.translation.x > max_player_x {
                player_transform.translation.x = max_player_x;
            }
        } 
        else if stage_progress.archers_killed < 3 || stage_progress.spearmen_killed < 4 {
            let max_player_x = 4200.0;
            if player_transform.translation.x > max_player_x {
                player_transform.translation.x = max_player_x;
            }
        }
    }
}

fn play_spearman_damage_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut query: Query<(&Spearman, &mut PreviousEnemyHp)>,
) {
    for (spearman, mut prev_hp) in query.iter_mut() {
      
        if spearman.hp < prev_hp.0 {
            if spearman.hp <= 0 {
              
                commands.spawn(AudioPlayer::new(asset_server.load("md3.mp3")));
            } else {
            
                commands.spawn(AudioPlayer::new(asset_server.load("md3.mp3")));
            }
        }
        prev_hp.0 = spearman.hp;
    }
}