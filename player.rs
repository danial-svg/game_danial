use bevy::prelude::*;
use crate::background::GameState;

#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub enum PlayerState {
    #[default]
    Idle,
    Running,
    DashRunning,
    Walking,
    Attacking,
    Jumping,
    DashJump,
    Dead,
}

#[derive(Component)]
pub struct Player {
    pub hp: i32,
    pub direction_facing_right: bool,
    pub current_state: PlayerState,
    pub ground_y: f32,
    pub altitude: f32,
    pub velocity_y: f32,
    pub is_grounded: bool,
    pub dash_jump_frame: usize,
}

#[derive(Component)]
pub struct PreviousHp(pub i32);

#[derive(Component)]
pub struct PreviousPlayerState(pub PlayerState);

#[derive(Component)]
pub struct HpBar;

#[derive(Resource)]
pub struct PlayerAnimationAssets {
    pub idle_texture: Handle<Image>,
    pub run_texture: Handle<Image>,
    pub walk_texture: Handle<Image>,
    pub attack_texture: Handle<Image>,
    pub dash_jump_textures: Vec<Handle<Image>>,
    pub death_texture: Handle<Image>,
    
    pub idle_layout: Handle<TextureAtlasLayout>,
    pub run_layout: Handle<TextureAtlasLayout>,
    pub walk_layout: Handle<TextureAtlasLayout>,
    pub attack_layout: Handle<TextureAtlasLayout>,
    pub death_layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
pub struct AnimationConfig {
    pub timer: Timer,
    pub frame_count: usize,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_player)
            .add_systems(
                Update,
                (
                    move_player, 
                    player_attack, 
                    apply_gravity,
                    animate_player,
                    check_death_condition,
                    update_hp_ui,
                    play_damage_sound,
                    play_attack_sound,
                )
                .run_if(in_state(GameState::InGame).or_else(in_state(GameState::NextLevel))),
            );
    }
}

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let hp_texture = asset_server.load("hp.png");
    let hp_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(350, 80),
        1, 
        6, 
        Some(UVec2::new(0, 33)),
        None
    ));

    commands.spawn((
        ImageNode {
            image: hp_texture,
            texture_atlas: Some(TextureAtlas {
                layout: hp_layout,
                index: 0,
            }),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            width: Val::Px(220.0),
            height: Val::Px(60.0),
            ..default()
        },
        HpBar,
    ));

    let idle_tex = asset_server.load("idle.png");
    let run_tex = asset_server.load("run.png");
    let walk_tex = asset_server.load("walk.png");
    let attack_tex = asset_server.load("attack.png");
    let death_tex = asset_server.load("death.png");

    let mut dash_jump_frames = Vec::new();
    for i in 1..=6 {
        dash_jump_frames.push(asset_server.load(format!("jump_{}.png", i)));
    }

    let idle_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(43, 62), 4, 1, Some(UVec2::new(85, 0)), None));
    let run_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(56, 68), 7, 1, Some(UVec2::new(70, 0)), None));
    let walk_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(53, 90), 8, 1, Some(UVec2::new(75, 0)), None));
    let attack_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(95, 76), 5, 1, Some(UVec2::new(28, 0)), None));
    let death_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(43, 62), 6, 1, Some(UVec2::new(85, 0)), None));

    commands.insert_resource(PlayerAnimationAssets {
        idle_texture: idle_tex.clone(),
        run_texture: run_tex,
        walk_texture: walk_tex,
        attack_texture: attack_tex,
        dash_jump_textures: dash_jump_frames,
        death_texture: death_tex,
        idle_layout: idle_layout.clone(),
        run_layout,
        walk_layout,
        attack_layout,
        death_layout,
    });

    commands.spawn((
        Sprite {
            image: idle_tex,
            texture_atlas: Some(TextureAtlas {
                layout: idle_layout,
                index: 0,
            }),
            custom_size: Some(Vec2::new(43.0, 62.0)), 
            ..default()
        },
        Transform::from_xyz(-600.0, -180.0, 5.0),
        Player {
            hp: 400,
            direction_facing_right: true,
            current_state: PlayerState::Idle,
            ground_y: -180.0,
            altitude: 0.0,
            velocity_y: 0.0,
            is_grounded: true,
            dash_jump_frame: 0,
        },
        PreviousHp(400), // اصلاح مقدار اولیه بر اساس HP واقعی بازیکن
        PreviousPlayerState(PlayerState::Idle),
        AnimationConfig {
            timer: Timer::from_seconds(0.12, TimerMode::Repeating),
            frame_count: 4,
        },
    ));
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>, 
    anim_assets: Res<PlayerAnimationAssets>,
    mut query: Query<(&mut Player, &mut Sprite, &mut AnimationConfig)>, 
) {
    let Ok((mut player, mut sprite, mut config)) = query.single_mut() else { return; };
    
    if player.current_state == PlayerState::Attacking || player.current_state == PlayerState::Dead {
        return;
    }

    let mut move_direction = Vec2::ZERO;
    let mut is_running = false;
    
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        move_direction.x -= 1.0;
        player.direction_facing_right = false;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        move_direction.x += 1.0;
        player.direction_facing_right = true;
    }
    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        move_direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        move_direction.y -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        is_running = true;
    }

    if keyboard_input.just_pressed(KeyCode::Space) && player.is_grounded {
        player.is_grounded = false;

        if player.current_state == PlayerState::Running || player.current_state == PlayerState::DashRunning {
            player.current_state = PlayerState::DashJump;
            player.dash_jump_frame = 0;
            player.velocity_y = 550.0; 
            
            sprite.image = anim_assets.dash_jump_textures[0].clone();
            sprite.texture_atlas = None; 
            sprite.custom_size = None; 
            
            config.frame_count = 6;
            config.timer.set_duration(std::time::Duration::from_secs_f32(0.08));
        } else {
            player.current_state = PlayerState::Jumping;
            player.velocity_y = 620.0; 
        }
    }

    if !player.is_grounded {
        sprite.flip_x = !player.direction_facing_right;
        return;
    }

    if move_direction != Vec2::ZERO {
        if is_running {
            if player.current_state != PlayerState::Running && player.current_state != PlayerState::DashRunning {
                player.current_state = PlayerState::Running;
                sprite.image = anim_assets.run_texture.clone();
                sprite.custom_size = Some(Vec2::new(56.0, 68.0)); 
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: anim_assets.run_layout.clone(),
                    index: 0,
                });
                config.frame_count = 7;
                config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
            }
        } else {
            if player.current_state != PlayerState::Walking {
                player.current_state = PlayerState::Walking;
                sprite.image = anim_assets.walk_texture.clone();
                sprite.custom_size = Some(Vec2::new(53.0, 90.0)); 
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: anim_assets.walk_layout.clone(),
                    index: 0,
                });
                config.frame_count = 8;
                config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
            }
        }
    } else {
        if player.current_state != PlayerState::Idle {
            player.current_state = PlayerState::Idle;
            sprite.image = anim_assets.idle_texture.clone();
            sprite.custom_size = Some(Vec2::new(43.0, 62.0)); 
            sprite.texture_atlas = Some(TextureAtlas {
                layout: anim_assets.idle_layout.clone(),
                index: 0,
                });
            config.frame_count = 4;
            config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
        }
    }

    sprite.flip_x = !player.direction_facing_right;
}

fn apply_gravity(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    anim_assets: Res<PlayerAnimationAssets>,
    mut query: Query<(&mut Transform, &mut Player, &mut Sprite, &mut AnimationConfig)>, 
    time: Res<Time>,
    stage_progress: Res<crate::background::StageProgress>,
    current_state: Res<State<GameState>>, 
) {
    let Ok((mut transform, mut player, mut sprite, mut config)) = query.single_mut() else { return; };
    if player.current_state == PlayerState::Dead { return; }

    let mut move_direction = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) { move_direction.x -= 1.0; }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) { move_direction.x += 1.0; }
    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) { move_direction.y += 1.0; }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) { move_direction.y -= 1.0; }

    let is_running = keyboard_input.pressed(KeyCode::ShiftLeft);
    
    let speed = if player.current_state == PlayerState::DashJump {
        550.0 
    } else if player.current_state == PlayerState::DashRunning {
        450.0 
    } else if is_running { 
        300.0 
    } else { 
        150.0 
    };

    if player.current_state == PlayerState::DashJump {
        let dir_sign = if player.direction_facing_right { 1.0 } else { -1.0 };
        transform.translation.x += dir_sign * speed * time.delta_secs();
        
        if move_direction.y != 0.0 {
            player.ground_y += move_direction.y * 150.0 * time.delta_secs();
        }
    } else if move_direction != Vec2::ZERO {
        let norm_dir = move_direction.normalize();
        player.ground_y += norm_dir.y * speed * time.delta_secs();
        transform.translation.x += norm_dir.x * speed * time.delta_secs();
    }

    if *current_state.get() == GameState::InGame {
        if stage_progress.archers_spawned == 1 && stage_progress.archers_killed == 0 {
            if transform.translation.x > 600.0 { transform.translation.x = 600.0; }
        } 
        else if stage_progress.spearmen_spawned == 1 && stage_progress.spearmen_killed == 0 {
            if transform.translation.x > 1800.0 { transform.translation.x = 1800.0; }
        } 
        else if stage_progress.bg33_reached && (stage_progress.archers_killed < 3 || stage_progress.spearmen_killed < 4) {
            if transform.translation.x > 2900.0 { transform.translation.x = 2900.0; }
        }
    }

    if !player.is_grounded {
        player.velocity_y -= 1700.0 * time.delta_secs();
        player.altitude += player.velocity_y * time.delta_secs();

        if player.altitude <= 0.0 {
            player.altitude = 0.0;
            player.velocity_y = 0.0;
            player.is_grounded = true;
            
            if player.current_state == PlayerState::DashJump {
                player.current_state = PlayerState::DashRunning;
                sprite.image = anim_assets.run_texture.clone();
                sprite.custom_size = Some(Vec2::new(56.0, 68.0)); 
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: anim_assets.run_layout.clone(),
                    index: 0,
                });
                config.frame_count = 7;
                config.timer.set_duration(std::time::Duration::from_secs_f32(0.10)); 
            } else if player.current_state == PlayerState::Jumping {
                player.current_state = PlayerState::Idle; 
            }
        }
    }

    let min_y = -240.0;
    let mut max_y = -10.0;

    if transform.translation.x >= 2500.0 && *current_state.get() == GameState::InGame {
        max_y = -140.0; 
    }

    player.ground_y = player.ground_y.clamp(min_y, max_y);
    let mut final_y = player.ground_y + player.altitude;

    if transform.translation.x >= 2500.0 && *current_state.get() == GameState::InGame {
        if final_y > 400.0 {
            final_y = 400.0;
            if !player.is_grounded && player.velocity_y > 0.0 {
                player.velocity_y = 0.0;
            }
        }
    }

    transform.translation.y = final_y;
    transform.translation.z = 10.0 - (player.ground_y / 100.0);

    if transform.translation.x < -600.0 {
        transform.translation.x = -600.0;
    }
}

fn player_attack(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    anim_assets: Res<PlayerAnimationAssets>,
    mut player_query: Query<(&mut Player, &mut Sprite, &mut AnimationConfig)>,
) {
    let Ok((mut player, mut sprite, mut config)) = player_query.single_mut() else { return; };
    if player.current_state == PlayerState::Dead { return; }

    if (mouse_input.just_pressed(MouseButton::Left) || keyboard_input.just_pressed(KeyCode::KeyJ)) 
        && player.current_state != PlayerState::Attacking 
    {
        player.current_state = PlayerState::Attacking;
        sprite.image = anim_assets.attack_texture.clone();
        sprite.custom_size = Some(Vec2::new(95.0, 76.0)); 
        sprite.texture_atlas = Some(TextureAtlas {
            layout: anim_assets.attack_layout.clone(),
            index: 0,
        });
        config.frame_count = 5;
        config.timer.set_duration(std::time::Duration::from_secs_f32(0.08));
    }
}

fn update_hp_ui(
    player_query: Query<&Player>,
    mut hp_bar_query: Query<&mut ImageNode, With<HpBar>>,
) {
    if let (Ok(player), Ok(mut image_node)) = (player_query.single(), hp_bar_query.single_mut()) {
        if let Some(atlas) = &mut image_node.texture_atlas {
            atlas.index = match player.hp {
                hp if hp >= 400 => 0,
                hp if hp >= 320  => 1,
                hp if hp >= 240  => 2,
                hp if hp >= 160  => 3,
                hp if hp >= 80  => 4,
                _               => 5,
            };
        }
    }
}

fn check_death_condition(
    anim_assets: Res<PlayerAnimationAssets>,
    mut query: Query<(&mut Player, &mut Sprite, &mut AnimationConfig)>,
) {
    let Ok((mut player, mut sprite, mut config)) = query.single_mut() else { return; };

    if player.hp <= 0 && player.current_state != PlayerState::Dead {
        player.current_state = PlayerState::Dead;
        sprite.image = anim_assets.death_texture.clone();
        sprite.custom_size = Some(Vec2::new(43.0, 62.0)); 
        sprite.texture_atlas = Some(TextureAtlas {
            layout: anim_assets.death_layout.clone(),
            index: 0,
        });
        config.frame_count = 6; 
        config.timer.set_duration(std::time::Duration::from_secs_f32(0.15));
        println!("بازیکن مرد!");
    }
}

fn animate_player(
    time: Res<Time>,
    anim_assets: Res<PlayerAnimationAssets>,
    mut query: Query<(&mut Player, &mut Sprite, &mut AnimationConfig)>,
) {
    let Ok((mut player, mut sprite, mut config)) = query.single_mut() else { return; };

    if config.frame_count > 1 {
        config.timer.tick(time.delta());
        if config.timer.just_finished() {
            
            let mut attack_finished = false;
            
            if player.current_state == PlayerState::Attacking {
                if let Some(atlas) = &sprite.texture_atlas {
                    if atlas.index >= config.frame_count - 1 {
                        attack_finished = true;
                    }
                }
            }

            if player.current_state == PlayerState::Dead {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    if atlas.index < config.frame_count - 1 {
                        atlas.index += 1;
                    }
                }
                return; 
            }

            if attack_finished {
                player.current_state = PlayerState::Idle;
                sprite.image = anim_assets.idle_texture.clone(); 
                sprite.custom_size = Some(Vec2::new(43.0, 62.0)); 
                config.frame_count = 4;
                config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
                
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: anim_assets.idle_layout.clone(),
                    index: 0,
                });
            } else {
                if player.current_state == PlayerState::DashJump {
                    if player.dash_jump_frame < config.frame_count - 1 {
                        player.dash_jump_frame += 1;
                        sprite.image = anim_assets.dash_jump_textures[player.dash_jump_frame].clone();
                    }
                } else {
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        if player.current_state == PlayerState::Attacking {
                            atlas.index += 1;
                        } else {
                            atlas.index = (atlas.index + 1) % config.frame_count;
                        }
                    }
                }
            }
        }
    }
}

fn play_damage_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut query: Query<(&Player, &mut PreviousHp)>,
) {
    if let Ok((player, mut prev_hp)) = query.single_mut() {
        if player.hp < prev_hp.0 {
            commands.spawn(AudioPlayer::new(asset_server.load("md1.mp3")));
        }
        prev_hp.0 = player.hp;
    }
}

fn play_attack_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut query: Query<(&Player, &mut PreviousPlayerState)>,
) {
    if let Ok((player, mut prev_state)) = query.single_mut() {
        if player.current_state == PlayerState::Attacking && prev_state.0 != PlayerState::Attacking {
            commands.spawn(AudioPlayer::new(asset_server.load("md4.mp3")));
        }
        prev_state.0 = player.current_state;
    }
}