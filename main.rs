use bevy::prelude::*;

mod background;
mod player;
mod archer;
mod spearman; 
mod swordsman; 
mod wizard; 
pub use player::{Player, PlayerState};
const PLAYER_IMG: &[u8] = include_bytes!("assets/player.png");
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) 
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        
        .add_plugins(background::BackgroundPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(archer::ArcherPlugin)
        .add_plugins(spearman::SpearmanPlugin)
        .add_plugins(swordsman::SwordsmanPlugin) 
        .add_plugins(wizard::WizardPlugin) 
        .run();
}
