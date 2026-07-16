use bevy::prelude::*;
use crate::Player;
use crate::PlayerState;
use crate::background::GameState;

pub struct SwordsmanPlugin;

impl Plugin for SwordsmanPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_swordsman)
            .add_systems(OnEnter(GameState::NextLevel), spawn_four_swordsmen)
            .add_systems(Update, (
                swordsman_ai,
                animate_swordsman,
                check_swordsman_death,
                player_attack_swordsman_collision,
            ).run_if(in_state(GameState::NextLevel)));
    }
}

#[derive(Component)]
struct Swordsman {
    hp: i32,
    direction_facing_right: bool,
    current_state: SwordsmanState,
    protect_timer: Timer,
    combo_index: u8,
    was_hit_by_player: bool,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum SwordsmanState {
    Idle,
    Walk,
    Protect,
    Attack1,
    Attack2,
    Attack3,
    Dead,
}

#[derive(Resource)]
struct SwordsmanAssets {
    idle_layout: Handle<TextureAtlasLayout>,
    walk_layout: Handle<TextureAtlasLayout>,
    protect_layout: Handle<TextureAtlasLayout>,
    attack1_layout: Handle<TextureAtlasLayout>,
    attack2_layout: Handle<TextureAtlasLayout>,
    attack3_layout: Handle<TextureAtlasLayout>,
    death_layout: Handle<TextureAtlasLayout>,
    
    idle_texture: Handle<Image>,
    walk_texture: Handle<Image>,
    protect_texture: Handle<Image>,
    attack1_texture: Handle<Image>,
    attack2_texture: Handle<Image>,
    attack3_texture: Handle<Image>,
    death_texture: Handle<Image>,
}

#[derive(Component)]
struct AnimationConfig {
    timer: Timer,
    frame_count: usize,
}

fn setup_swordsman(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let idle_texture = asset_server.load("Skeleton_Warrior/Idle.png");
    let idle_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 7, 1, None, None));

    let walk_texture = asset_server.load("Skeleton_Warrior/Walk.png");
    let walk_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 7, 1, None, None));

    let protect_texture = asset_server.load("Skeleton_Warrior/Protect.png");
    let protect_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 1, 1, None, None));

    let attack1_texture = asset_server.load("Skeleton_Warrior/Attack_1.png");
    let attack1_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 5, 1, None, None));

    let attack2_texture = asset_server.load("Skeleton_Warrior/Attack_2.png");
    let attack2_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 6, 1, None, None));

    let attack3_texture = asset_server.load("Skeleton_Warrior/Attack_3.png");
    let attack3_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 4, 1, None, None));

    let death_texture = asset_server.load("Skeleton_Warrior/Dead.png");
    let death_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 4, 1, None, None));

    // اصلاح نام فیلدها در خطوط پایین (اضافه شدن شناسه صحیح متغیرها)
    commands.insert_resource(SwordsmanAssets {
        idle_layout,
        walk_layout,
        protect_layout,
        attack1_layout,
        attack2_layout,
        attack3_layout,
        death_layout,
        idle_texture,
        walk_texture,
        protect_texture,
        attack1_texture,
        attack2_texture,
        attack3_texture,
        death_texture,
    });
}

fn spawn_four_swordsmen(
    mut commands: Commands,
    assets: Res<SwordsmanAssets>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    if let Ok(mut player_transform) = player_query.single_mut() {
        player_transform.translation.x = 0.0;
        player_transform.translation.y = -180.0;
    }

    let spawn_x_positions = [350.0, 550.0, 750.0, 950.0];

    for x_pos in spawn_x_positions {
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
            Swordsman {
                hp: 100,
                direction_facing_right: false,
                current_state: SwordsmanState::Idle,
                protect_timer: Timer::from_seconds(1.5, TimerMode::Once),
                combo_index: 1,
                was_hit_by_player: false,
            },
            AnimationConfig {
                timer: Timer::from_seconds(0.12, TimerMode::Repeating),
                frame_count: 7,
            },
        ));
    }
    println!("-> تعداد ۴ شمشیرزن اسکلتی جدید در فواصل مختلف نقشه متولد شدند!");
}

fn swordsman_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut swordsman_query: Query<(&mut Swordsman, &mut Transform, &mut Sprite, &mut AnimationConfig), Without<Player>>,
    swordsman_assets: Res<SwordsmanAssets>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut swordsman, mut swordsman_transform, mut sprite, mut config) in swordsman_query.iter_mut() {
            if swordsman.current_state == SwordsmanState::Dead { continue; }

            if swordsman.current_state == SwordsmanState::Protect {
                swordsman.protect_timer.tick(time.delta());
                if swordsman.protect_timer.just_finished() {
                    swordsman.current_state = SwordsmanState::Idle;
                    swordsman.combo_index = 1;
                    sprite.image = swordsman_assets.idle_texture.clone();
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.layout = swordsman_assets.idle_layout.clone();
                        atlas.index = 0;
                    }
                    config.frame_count = 7;
                    config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
                }
                continue;
            }

            if matches!(swordsman.current_state, SwordsmanState::Attack1 | SwordsmanState::Attack2 | SwordsmanState::Attack3) {
                continue;
            }

            let player_pos = player_transform.translation;
            let swordsman_pos = swordsman_transform.translation;
            
            let x_direction = player_pos.x - swordsman_pos.x;
            let x_distance = x_direction.abs();
            let y_direction = player_pos.y - swordsman_pos.y;
            let y_distance = y_direction.abs();

            if x_direction > 0.0 {
                swordsman.direction_facing_right = true;
                sprite.flip_x = false;
            } else {
                swordsman.direction_facing_right = false;
                sprite.flip_x = true;
            }

            if y_distance > 10.0 {
                swordsman_transform.translation.y += y_direction.signum() * 80.0 * time.delta_secs();
            }

            if x_distance < 50.0 && y_distance < 25.0 {
                swordsman.current_state = SwordsmanState::Attack1;
                swordsman.combo_index = 1;
                sprite.image = swordsman_assets.attack1_texture.clone();
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.layout = swordsman_assets.attack1_layout.clone();
                    atlas.index = 0;
                }
                config.frame_count = 5;
                config.timer.set_duration(std::time::Duration::from_secs_f32(0.09));
               
            } else {
                swordsman_transform.translation.x += x_direction.signum() * 90.0 * time.delta_secs();
                if swordsman.current_state != SwordsmanState::Walk {
                    swordsman.current_state = SwordsmanState::Walk;
                    sprite.image = swordsman_assets.walk_texture.clone();
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.layout = swordsman_assets.walk_layout.clone();
                        atlas.index = 0;
                    }
                    config.frame_count = 7;
                    config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
                }
            }
        }
    }
}

fn animate_swordsman(
    time: Res<Time>,
    mut player_query: Query<&mut Player>,
    mut query: Query<(&mut Swordsman, &mut Sprite, &mut AnimationConfig)>,
    swordsman_assets: Res<SwordsmanAssets>,
) {
    if let Ok(mut player_res) = player_query.single_mut() {
        for (mut swordsman, mut sprite, mut config) in query.iter_mut() {
            if swordsman.current_state == SwordsmanState::Protect { continue; }

            config.timer.tick(time.delta());
            if config.timer.just_finished() {
                let mut combo_finished = false;

                if let Some(atlas) = &mut sprite.texture_atlas {
                    match swordsman.current_state {
                        SwordsmanState::Attack1 | SwordsmanState::Attack2 | SwordsmanState::Attack3 => {
                            if atlas.index < config.frame_count - 1 {
                                atlas.index += 1;
                            } else {
                                combo_finished = true;
                            }
                        }
                        SwordsmanState::Dead => {
                            if atlas.index < config.frame_count - 1 {
                                atlas.index += 1;
                            }
                        }
                        _ => {
                            atlas.index = (atlas.index + 1) % config.frame_count;
                        }
                    }
                }

                if combo_finished {
                    if player_res.hp > 0 { player_res.hp -= 10; }
                    println!(" شمشیر زن به شما ضربه زد {}", player_res.hp);

                    match swordsman.combo_index {
                        1 => {
                            swordsman.combo_index = 2;
                            swordsman.current_state = SwordsmanState::Attack2;
                            sprite.image = swordsman_assets.attack2_texture.clone();
                            if let Some(atlas) = &mut sprite.texture_atlas {
                                atlas.layout = swordsman_assets.attack2_layout.clone();
                                atlas.index = 0;
                            }
                            config.frame_count = 6;
                            config.timer.set_duration(std::time::Duration::from_secs_f32(0.09));
                        }
                        2 => {
                            swordsman.combo_index = 3;
                            swordsman.current_state = SwordsmanState::Attack3;
                            sprite.image = swordsman_assets.attack3_texture.clone();
                            if let Some(atlas) = &mut sprite.texture_atlas {
                                atlas.layout = swordsman_assets.attack3_layout.clone();
                                atlas.index = 0;
                            }
                            config.frame_count = 4;
                            config.timer.set_duration(std::time::Duration::from_secs_f32(0.09));
                        }
                        _ => {
                            swordsman.combo_index = 1;
                            swordsman.current_state = SwordsmanState::Idle;
                            sprite.image = swordsman_assets.idle_texture.clone();
                            if let Some(atlas) = &mut sprite.texture_atlas {
                                atlas.layout = swordsman_assets.idle_layout.clone();
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

fn player_attack_swordsman_collision(
    player_query: Query<(&Transform, &Player)>,
    mut swordsman_query: Query<(&mut Swordsman, &Transform, &mut Sprite, &mut AnimationConfig)>,
    swordsman_assets: Res<SwordsmanAssets>,
) {
    if let Ok((player_transform, player)) = player_query.single() {
        for (mut swordsman, swordsman_transform, mut sprite, mut config) in swordsman_query.iter_mut() {
            if swordsman.current_state == SwordsmanState::Dead { continue; }

            if player.current_state == PlayerState::Attacking {
                let player_pos = player_transform.translation;
                let swordsman_pos = swordsman_transform.translation;

                let x_distance = (player_pos.x - swordsman_pos.x).abs();
                let y_distance = (player_pos.y - swordsman_pos.y).abs();

                if x_distance < 65.0 && y_distance < 20.0 {
                    if !swordsman.was_hit_by_player {
                        swordsman.was_hit_by_player = true;

                        let is_player_in_front = (swordsman.direction_facing_right && player_pos.x > swordsman_pos.x) 
                            || (!swordsman.direction_facing_right && player_pos.x < swordsman_pos.x);

                        if swordsman.current_state == SwordsmanState::Protect {
                            if is_player_in_front {
                                println!(" ضربه شما توسط سپر اسکلت کاملاً دفع شد!");
                            } else {
                                swordsman.hp -= 20;
                            }
                        } else {
                            swordsman.hp -= 20;
                           
                            if swordsman.hp > 0 {
                                swordsman.current_state = SwordsmanState::Protect;
                                swordsman.protect_timer.reset();
                                sprite.image = swordsman_assets.protect_texture.clone();
                                if let Some(atlas) = &mut sprite.texture_atlas {
                                    atlas.layout = swordsman_assets.protect_layout.clone();
                                    atlas.index = 0;
                                }
                                config.frame_count = 1;
                                config.timer.set_duration(std::time::Duration::from_secs_f32(1.5));
                            }
                        }
                    }
                }
            } else {
                swordsman.was_hit_by_player = false;
            }
        }
    }
}

fn check_swordsman_death(
    anim_assets: Res<SwordsmanAssets>,
    mut query: Query<(&mut Swordsman, &mut Sprite, &mut AnimationConfig)>,
) {
    for (mut swordsman, mut sprite, mut config) in query.iter_mut() {
        if swordsman.hp <= 0 && swordsman.current_state != SwordsmanState::Dead {
            swordsman.current_state = SwordsmanState::Dead;
            println!("شمشیر زن نابود شد!");
            
            sprite.image = anim_assets.death_texture.clone();
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.layout = anim_assets.death_layout.clone();
                atlas.index = 0;
            }
            config.frame_count = 4;
            config.timer.set_duration(std::time::Duration::from_secs_f32(0.15));
        }
    }
}