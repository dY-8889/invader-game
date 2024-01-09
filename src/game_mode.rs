use bevy::ui::Val;

mod single_play;
mod tow_player;
mod vs_player;

pub use single_play::*;
pub use tow_player::*;
pub use vs_player::*;

pub const TEXT_PADDING: Val = Val::Px(7.0);
