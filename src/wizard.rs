use bevy::prelude::*;
use crate::background::GameState;
use crate::{Player, PlayerState};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum WizardState {
    Idle,        
    Walking,      
    KnifeAttack,  
    CastFireball, 
    FlameBreath,  
    Fleeing,     
    Hurt,       
    Dead,        
}

#[derive(Component)]
pub struct Wizard {
    pub hp: i32,
    pub max_hp: i32,
    pub state: WizardState,
    pub speed: f32,
    pub direction_facing_left: bool,
    pub first_hit_taken: bool, 
    pub flame_triggered: bool, 
    pub attack_timer: Timer,
    pub consecutive_hits: u32, 
}

#[derive(Component)]
pub struct WizardAnimationConfig {
    pub timer: Timer,
    pub frame_count: usize,
}

#[derive(Component)]
pub struct FireballProjectile {
    pub direction_left: bool,
    pub speed: f32,
}

#[derive(Resource)]
pub struct WizardAssets {
    pub idle_texture: Handle<Image>,
    pub walk_texture: Handle<Image>,
    pub run_texture: Handle<Image>, 
    pub attack1_texture: Handle<Image>, 
    pub fireball_texture: Handle<Image>, 
    pub chargo_texture: Handle<Image>, 
    pub flame_texture: Handle<Image>, 
    pub hurt_texture: Handle<Image>,
    pub dead_texture: Handle<Image>,

    pub idle_layout: Handle<TextureAtlasLayout>,
    pub walk_layout: Handle<TextureAtlasLayout>,
    pub run_layout: Handle<TextureAtlasLayout>, 
    pub attack1_layout: Handle<TextureAtlasLayout>,
    pub fireball_layout: Handle<TextureAtlasLayout>,
    pub chargo_layout: Handle<TextureAtlasLayout>,
    pub flame_layout: Handle<TextureAtlasLayout>,
    pub hurt_layout: Handle<TextureAtlasLayout>,
    pub dead_layout: Handle<TextureAtlasLayout>,
}

pub struct WizardPlugin;

impl Plugin for WizardPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::NextLevel), setup_wizard_resources_and_spawn)
            .add_systems(
                Update,
                (
                    wizard_ai,
                    move_projectiles,
                    animate_wizard,
                    animate_projectiles,
                    check_wizard_death_test,
                    damage_player_with_fireball,       
                    damage_wizard_with_player_attack,  
                )
                .run_if(in_state(GameState::NextLevel)), 
            );
    }
}

fn setup_wizard_resources_and_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let idle_tex = asset_server.load("Fire Wizard/Idle.png");
    let walk_tex = asset_server.load("Fire Wizard/Walk.png");
    let run_tex = asset_server.load("Fire Wizard/Run.png"); 
    let attack1_tex = asset_server.load("Fire Wizard/Attack1.png");
    let fireball_tex = asset_server.load("Fire Wizard/Fireball.png");
    let chargo_tex = asset_server.load("Fire Wizard/Chargo.png");
    let flame_tex = asset_server.load("Fire Wizard/Flame.png");
    let hurt_tex = asset_server.load("Fire Wizard/Hurt.png");
    let dead_tex = asset_server.load("Fire Wizard/Dead.png");

    let idle_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 7, 1, None, None)); 
    let walk_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 6, 1, None, None)); 
    let run_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 8, 1, None, None)); 
    let attack1_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 4, 1, None, None)); 
    let fireball_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 8, 1, None, None)); 
    let chargo_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(64, 64), 12, 1, None, None)); 
    let flame_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 14, 1, None, None)); 
    let hurt_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 3, 1, None, None)); 
    let dead_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 6, 1, None, None)); 

    let wizard_assets = WizardAssets {
        idle_texture: idle_tex.clone(),
        walk_texture: walk_tex,
        run_texture: run_tex, 
        attack1_texture: attack1_tex,
        fireball_texture: fireball_tex,
        chargo_texture: chargo_tex,
        flame_texture: flame_tex,
        hurt_texture: hurt_tex,
        dead_texture: dead_tex,
        idle_layout: idle_layout.clone(),
        walk_layout,
        run_layout, 
        attack1_layout,
        fireball_layout,
        chargo_layout,
        flame_layout,
        hurt_layout,
        dead_layout,
    };

    commands.insert_resource(wizard_assets);

    commands.spawn((
        Sprite {
            image: idle_tex,
            texture_atlas: Some(TextureAtlas {
                layout: idle_layout,
                index: 0,
            }),
            custom_size: Some(Vec2::new(128.0, 128.0)),
            ..default()
        },
        Transform::from_xyz(400.0, -180.0, 6.0), 
        Wizard {
            hp: 150,
            max_hp: 150,
            state: WizardState::Idle,
            speed: 80.0,
            direction_facing_left: true,
            first_hit_taken: false,
            flame_triggered: false,
            attack_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            consecutive_hits: 0,
        },
        WizardAnimationConfig {
            timer: Timer::from_seconds(0.12, TimerMode::Repeating),
            frame_count: 7,
        },
    ));
}

fn wizard_ai(
    mut commands: Commands,
    time: Res<Time>,
    wizard_assets: Option<Res<WizardAssets>>, 
    player_query: Query<&Transform, With<Player>>,
    mut wizard_query: Query<(&mut Transform, &mut Wizard, &mut Sprite, &mut WizardAnimationConfig), Without<Player>>,
) {
    let Some(assets) = wizard_assets else { return; };
    let Some(player_transform) = player_query.iter().next() else { return; }; 
    
    for (mut wiz_transform, mut wizard, mut sprite, mut config) in wizard_query.iter_mut() {
        if wizard.state == WizardState::Dead || wizard.state == WizardState::Hurt {
            continue;
        }

        if wizard.hp < wizard.max_hp && !wizard.first_hit_taken && !wizard.flame_triggered {
            wizard.first_hit_taken = true;
            wizard.flame_triggered = true;
            wizard.state = WizardState::FlameBreath;
            sprite.image = assets.flame_texture.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: assets.flame_layout.clone(),
                index: 0,
            });
            config.frame_count = 14;
            config.timer.set_duration(std::time::Duration::from_secs_f32(0.05));
            continue;
        }

        if wizard.state == WizardState::KnifeAttack || wizard.state == WizardState::CastFireball || wizard.state == WizardState::FlameBreath {
            continue;
        }

        let diff_x = player_transform.translation.x - wiz_transform.translation.x;
        let diff_y = player_transform.translation.y - wiz_transform.translation.y; 
        
        let distance_x = diff_x.abs();
        let distance_y = diff_y.abs();

        if wizard.state == WizardState::Fleeing {
            if distance_x >= 650.0 {
                wizard.state = WizardState::Idle;
                continue;
            } else {
                let flee_dir_x = if diff_x > 0.0 { -1.0 } else { 1.0 };
                wizard.direction_facing_left = flee_dir_x < 0.0;
                sprite.flip_x = wizard.direction_facing_left;

                wiz_transform.translation.x += flee_dir_x * (wizard.speed * 2.8) * time.delta_secs();
                continue;
            }
        }

        wizard.direction_facing_left = diff_x < 0.0;
        sprite.flip_x = wizard.direction_facing_left;

        if distance_x < 120.0 {
            wizard.state = WizardState::FlameBreath;
            sprite.image = assets.flame_texture.clone(); 
            sprite.texture_atlas = Some(TextureAtlas {
                layout: assets.flame_layout.clone(),
                index: 0,
            });
            config.frame_count = 14; 
            config.timer.set_duration(std::time::Duration::from_secs_f32(0.08)); // سرعت انیمیشن آتش
            continue;
        }

        if distance_y > 10.0 {
            wizard.state = WizardState::Walking;
            sprite.image = assets.walk_texture.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: assets.walk_layout.clone(),
                index: 0,
            });
            config.frame_count = 6;
            config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
            
            let move_dir_y = if diff_y > 0.0 { 1.0 } else { -1.0 };
            wiz_transform.translation.y += move_dir_y * wizard.speed * time.delta_secs();
            continue;
        }

        if distance_x < 65.0 {
            wizard.state = WizardState::KnifeAttack;
            sprite.image = assets.attack1_texture.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: assets.attack1_layout.clone(),
                index: 0,
            });
            config.frame_count = 4;
            config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
        }
        else if distance_x < 350.0 && distance_x >= 180.0 {
            wizard.attack_timer.tick(time.delta());
            if wizard.attack_timer.just_finished() {
                wizard.state = WizardState::CastFireball;
                sprite.image = assets.fireball_texture.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: assets.fireball_layout.clone(), 
                    index: 0,
                });
                config.frame_count = 8;
                config.timer.set_duration(std::time::Duration::from_secs_f32(0.06));

                let spawn_offset = if wizard.direction_facing_left { -40.0 } else { 40.0 };
                commands.spawn((
                    Sprite {
                        image: assets.chargo_texture.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: assets.chargo_layout.clone(),
                            index: 0,
                        }),
                        custom_size: Some(Vec2::new(64.0, 64.0)),
                        flip_x: wizard.direction_facing_left,
                        ..default()
                    },
                    Transform::from_xyz(
                        wiz_transform.translation.x + spawn_offset,
                        wiz_transform.translation.y + 10.0,
                        wiz_transform.translation.z + 1.0
                    ),
                    FireballProjectile {
                        direction_left: wizard.direction_facing_left,
                        speed: 700.0,
                    },
                    WizardAnimationConfig {
                        timer: Timer::from_seconds(0.04, TimerMode::Repeating),
                        frame_count: 12,
                    },
                ));
            } else {
                wizard.state = WizardState::Idle;
                sprite.image = assets.idle_texture.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: assets.idle_layout.clone(),
                    index: 0,
                });
                config.frame_count = 7;
                config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
            }
        }
        else {
            set_wizard_walking(&mut wizard, &mut sprite, &mut config, &assets, &mut wiz_transform, diff_x, time.delta_secs());
        }
    }
}

/// **تابع کمکی راه رفتن:**
fn set_wizard_walking(
    wizard: &mut Wizard,
    sprite: &mut Sprite,
    config: &mut WizardAnimationConfig,
    assets: &WizardAssets,
    transform: &mut Transform,
    diff: f32,
    delta: f32,
) {
    wizard.state = WizardState::Walking;
    sprite.image = assets.walk_texture.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: assets.walk_layout.clone(),
        index: 0,
    });
    config.frame_count = 6;
    config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));

    let move_dir = if diff > 0.0 { 1.0 } else { -1.0 };
    transform.translation.x += move_dir * wizard.speed * delta;
}

fn animate_wizard(
    time: Res<Time>,
    wizard_assets: Option<Res<WizardAssets>>, 
    mut query: Query<(&mut Wizard, &mut Sprite, &mut WizardAnimationConfig), With<Wizard>>,
) {
    let Some(assets) = wizard_assets else { return; };
    
    for (mut wizard, mut sprite, mut config) in query.iter_mut() {
        config.timer.tick(time.delta());
        if config.timer.just_finished() {
            if wizard.state == WizardState::Dead {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    if atlas.index < config.frame_count - 1 {
                        atlas.index += 1;
                    }
                }
                continue;
            }

            let mut animation_finished = false;
            if let Some(atlas) = &mut sprite.texture_atlas {
                if atlas.index >= config.frame_count - 1 {
                    animation_finished = true;
                } else {
                    atlas.index += 1;
                }
            }

            if animation_finished {
                if wizard.state == WizardState::FlameBreath {
                    wizard.state = WizardState::Idle;
                    sprite.image = assets.idle_texture.clone();
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: assets.idle_layout.clone(),
                        index: 0,
                    });
                    config.frame_count = 7;
                    config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
                    continue;
                }

                if wizard.state == WizardState::Hurt && wizard.consecutive_hits >= 2 {
                    wizard.consecutive_hits = 0; 
                    wizard.state = WizardState::Fleeing;
                    sprite.image = assets.run_texture.clone(); 
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: assets.run_layout.clone(),
                        index: 0,
                    });
                    config.frame_count = 8; 
                    config.timer.set_duration(std::time::Duration::from_secs_f32(0.10));
                    continue;
                }
                
                wizard.state = WizardState::Idle;
                sprite.image = assets.idle_texture.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: assets.idle_layout.clone(),
                    index: 0,
                });
                config.frame_count = 7;
                config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
            }
        }
    }
}

fn check_wizard_death_test(
    wizard_assets: Option<Res<WizardAssets>>,
    mut query: Query<(&mut Wizard, &mut Sprite, &mut WizardAnimationConfig)>,
) {
    let Some(assets) = wizard_assets else { return; };
    for (mut wizard, mut sprite, mut config) in query.iter_mut() {
        if wizard.hp <= 0 && wizard.state != WizardState::Dead {
            wizard.state = WizardState::Dead;
            sprite.image = assets.dead_texture.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: assets.dead_layout.clone(),
                index: 0,
            });
            config.frame_count = 6;
            config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
        }
    }
}

fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &FireballProjectile)>,
) {
    for (entity, mut transform, projectile) in query.iter_mut() {
        let dir = if projectile.direction_left { -1.0 } else { 1.0 };
        transform.translation.x += dir * projectile.speed * time.delta_secs();

        if transform.translation.x.abs() > 3000.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn animate_projectiles(
    mut commands: Commands, 
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut WizardAnimationConfig), With<FireballProjectile>>, 
) {
    for (entity, mut sprite, mut config) in query.iter_mut() {
        config.timer.tick(time.delta());
        if config.timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                if atlas.index >= config.frame_count - 1 {
                    commands.entity(entity).despawn(); 
                } else {
                    atlas.index += 1;
                }
            }
        }
    }
}

fn damage_player_with_fireball(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut Player)>, 
    projectile_query: Query<(Entity, &Transform), With<FireballProjectile>>,
) {
    let Ok((player_transform, mut player)) = player_query.single_mut() else { return; };

    for (proj_entity, proj_transform) in projectile_query.iter() {
        let distance = player_transform.translation.distance(proj_transform.translation);
        
        if distance < 40.0 {
            player.hp -= 10; 
            println!("💥 فایربال به بازیکن برخورد کرد! خون بازیکن: {}", player.hp);
            commands.entity(proj_entity).despawn();
        }
    }
}

fn damage_wizard_with_player_attack(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>, 
    player_query: Query<(&Transform, &Player)>, 
    mut wizard_query: Query<(&Transform, &mut Wizard, &mut Sprite, &mut WizardAnimationConfig)>,
    wizard_assets: Option<Res<WizardAssets>>,
) {
    let Some(assets) = wizard_assets else { return; };
    let Ok((player_transform, player)) = player_query.single() else { return; };
    
    if (mouse_input.just_pressed(MouseButton::Left) || keyboard_input.just_pressed(KeyCode::KeyJ)) 
        || player.current_state == PlayerState::Attacking 
    { 
        for (wiz_transform, mut wizard, mut sprite, mut config) in wizard_query.iter_mut() {
            let distance = player_transform.translation.distance(wiz_transform.translation);
            
            if distance < 70.0 && wizard.state != WizardState::Dead && wizard.state != WizardState::Hurt {
                wizard.hp -= 25; 
                wizard.consecutive_hits += 1; 
                println!("⚔️ به جادوگر آسیب زدید! ضربه پیاپی: {}/2 | خون باقی‌مانده: {}", wizard.consecutive_hits, wizard.hp);
                
                if wizard.hp > 0 {
                    wizard.state = WizardState::Hurt;
                    sprite.image = assets.hurt_texture.clone();
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: assets.hurt_layout.clone(),
                        index: 0,
                    });
                    config.frame_count = 3;
                    config.timer.set_duration(std::time::Duration::from_secs_f32(0.12));
                    config.timer.reset(); 
                }
            }
        }
    }
}