use bevy::prelude::*;

use super::GameMode;
use crate::menu::{BUTTON_HEIGHT, BUTTON_WIDTH};
use crate::FontResource;

#[derive(Component)]
pub struct GameMenuScreen;

pub fn game_menu_setup(mut commands: Commands, font: Res<FontResource>) {
    let button_style = Style {
        width: BUTTON_WIDTH,
        height: BUTTON_HEIGHT,
        margin: UiRect::all(Val::Px(20.)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let text_style = |font_size| TextStyle {
        font: font.0.clone(),
        font_size,
        color: Color::BLUE,
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            GameMenuScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Game Mode", text_style(100.)));

                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                ..default()
                            },
                            GameMode::Single,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Single", text_style(60.)));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                ..default()
                            },
                            GameMode::Tow,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Tow Play", text_style(60.)));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                ..default()
                            },
                            GameMode::VS,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Online Play", text_style(60.)));
                        });
                });
        });
}

pub fn game_menu_system(
    interaction: Query<(&Interaction, &GameMode), (Changed<Interaction>, With<Button>)>,
    mut game_mode: ResMut<NextState<GameMode>>,
) {
    for (interaction, mode) in &interaction {
        if *interaction == Interaction::Pressed {
            game_mode.set(*mode);
        }
    }
}
