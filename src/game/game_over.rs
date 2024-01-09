use bevy::{app::AppExit, prelude::*};

use crate::entity::EnemyCollider;
use crate::menu::MenuState;
use crate::{despawn_screen, FontResource, MainState};

use super::{GameMode, PlayerInfoScreen};

const BUTTON_INFO: [(&str, GameOverButtonAction); 3] = [
    ("Replay", GameOverButtonAction::Replay),
    ("Menu", GameOverButtonAction::Menu),
    ("Quit", GameOverButtonAction::Quit),
];

pub struct GameOver;

impl Plugin for GameOver {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MainState::GameOver), game_over_setup)
            .add_systems(
                OnExit(MainState::GameOver),
                despawn_screen::<GameOverScreen>,
            )
            .add_systems(
                Update,
                game_over_button_system.run_if(in_state(MainState::GameOver)),
            );
    }
}

fn game_over_setup(mut commands: Commands, font: Res<FontResource>) {
    let text_style = |size| TextStyle {
        font: font.0.clone(),
        font_size: size,
        color: Color::RED,
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            GameOverScreen,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Game Over", text_style(140.)));

            for (text, action) in BUTTON_INFO {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(220.),
                                height: Val::Px(70.),
                                margin: UiRect::vertical(Val::Px(30.)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            ..default()
                        },
                        action,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(text, text_style(60.)));
                    });
            }
        });
}

#[derive(Component)]
struct GameOverScreen;

#[derive(Component)]
enum GameOverButtonAction {
    Replay,
    Menu,
    Quit,
}

fn game_over_button_system(
    interaction_query: Query<
        (&Interaction, &GameOverButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut main_state: ResMut<NextState<MainState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut app_exit_event: EventWriter<AppExit>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match *action {
                GameOverButtonAction::Replay => main_state.set(MainState::Game),
                GameOverButtonAction::Menu => {
                    main_state.set(MainState::Menu);
                    menu_state.set(MenuState::Main);
                }
                GameOverButtonAction::Quit => app_exit_event.send(AppExit),
            }
        }
    }
}

pub fn entity_despawn<E: Component, Attack: Component>(
    mut commands: Commands,
    entity_query: Query<
        Entity,
        Or<(
            With<E>,
            With<Attack>,
            With<EnemyCollider>,
            With<PlayerInfoScreen>,
            // With<SinglePlayer>,
            // With<SinglePlayerAttack>,
            // With<Player1>,
            // With<PlayerAttack1>,
            // With<Player2>,
            // With<PlayerAttack2>,
            // With<My>,
            // With<MyAttack>,
            // With<Opponent>,
            // With<OpponentAttack>,
        )>,
    >,
    mut game_mode: ResMut<NextState<GameMode>>,
) {
    game_mode.set(GameMode::Disabled);

    for entity in &entity_query {
        commands.entity(entity).despawn_recursive()
    }
}
