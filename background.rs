use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    InGame,     
    NextLevel,  
}

#[derive(Component)]
pub struct BackgroundLayer {
    pub id: u32,
}

#[derive(Resource)]
pub struct StageProgress {
    pub archers_spawned: u32,
    pub archers_killed: u32,
    pub spearmen_spawned: u32,
    pub spearmen_killed: u32,
    pub bg33_reached: bool,
    pub wave_3_spawned: bool,
    pub bg4_active: bool, 
}

impl Default for StageProgress {
    fn default() -> Self {
        Self {
            archers_spawned: 0,
            archers_killed: 0,
            spearmen_spawned: 0,
            spearmen_killed: 0,
            bg33_reached: false,
            wave_3_spawned: false,
            bg4_active: false, 
        }
    }
}

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameState>() 
            .init_resource::<StageProgress>()
            .add_systems(Startup, setup_background)
            .add_systems(
                Update, 
                (camera_follow, update_stage_logic).run_if(in_state(GameState::InGame))
            )
            .add_systems(OnEnter(GameState::NextLevel), setup_next_level_background);
    }
}

fn setup_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d::default());

    commands.spawn(AudioPlayer::new(asset_server.load("onetent-mortal-gaming-144000.mp3")))
        .insert(PlaybackSettings::LOOP);

    let bg_width = 1280.0;

    for id in 1..=3 {
        let tex_name = format!("Battleground{}.png", id);
        commands.spawn((
            Sprite {
                image: asset_server.load(tex_name),
                custom_size: Some(Vec2::new(bg_width, 720.0)),
                ..default()
            },
            Transform::from_xyz((id - 1) as f32 * bg_width, 0.0, 0.0),
            BackgroundLayer { id },
        ));
    }
}

fn setup_next_level_background(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    bg_query: Query<Entity, With<BackgroundLayer>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    // اصلاح متد حذف بازگشتی در Bevy
    for entity in bg_query.iter() {
        commands.entity(entity).despawn();
    }

    // اصلاح ساختار دریافت دوربین
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        camera_transform.translation.x = 0.0;
    }

    let bg_width = 1280.0;

    commands.spawn((
        Sprite {
            image: asset_server.load("Battleground5.png"),
            custom_size: Some(Vec2::new( bg_width, 720.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        BackgroundLayer { id: 5 },
    ));
    println!("-> بک‌گراند ۵ لود شد و موقعیت دوربین ریست گردید.");
}

fn camera_follow(
    player_query: Query<&Transform, With<crate::Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<crate::Player>)>,
    stage_progress: Res<StageProgress>,
) {
    // اصلاح ساختار دریافت کامپوننت‌های تک‌عضوی
    if let (Ok(player_transform), Ok(mut camera_transform)) = (player_query.single(), camera_query.single_mut()) {
        let target_x = player_transform.translation.x;
        let mut max_camera_x = 0.0;

        if stage_progress.archers_killed >= 1 {
            max_camera_x = 1300.0;
        }

        if stage_progress.spearmen_killed >= 1 {
            max_camera_x = 2400.0;
        }

        camera_transform.translation.x = target_x.clamp(0.0, max_camera_x);
    }
}

fn update_stage_logic(
    player_query: Query<&Transform, With<crate::Player>>,
    mut bg_query: Query<(&mut Sprite, &mut Transform, &BackgroundLayer), Without<crate::Player>>,
    mut stage_progress: ResMut<StageProgress>,
    mut next_state: ResMut<NextState<GameState>>, 
    asset_server: Res<AssetServer>,
) {
    // اصلاح دریافت بازیکن
    if let Ok(player_transform) = player_query.single() {
        let player_x = player_transform.translation.x;
        let bg_width = 1280.0;
        let player_y =player_transform.translation.y;

        if player_x >= 2600.0 && !stage_progress.bg33_reached {
            stage_progress.bg33_reached = true;
            println!("-> بازیکن به بک‌گراند ۳ رسید! آماده‌سازی اسپاون موج عقب...");
        }

        if stage_progress.archers_killed >= 3 && stage_progress.spearmen_killed >= 4 {
            for (mut sprite, mut transform, bg) in bg_query.iter_mut() {
                if bg.id == 3 && sprite.image != asset_server.load("Battleground4.png") {
                    sprite.image = asset_server.load("Battleground4.png");
                    transform.translation.x = 2.0 * bg_width;
                    stage_progress.bg4_active = true; 
                    println!("-> همه ۵ دشمن موج آخر کشته شدند! بک‌گراند ۴ جایگزین بک‌گراند ۳ شد.");
                }
            }
        }

        if player_x >= 2600.0 && player_x <= 2700.0&&player_y >-80.0 &&stage_progress.bg4_active {
            println!("-> بازیکن در وضعیت بک‌گراند ۴ به نقطه ۲۵۰۰ رسید! انتقال به مرحله بعد...");
            next_state.set(GameState::NextLevel);
        }
    }
}