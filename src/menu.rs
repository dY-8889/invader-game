use bevy::{app::AppExit, prelude::*};

use crate::{despawn_screen, FontResource, MainState};

pub const BUTTON_WIDTH: Val = Val::Px(250.);
pub const BUTTON_HEIGHT: Val = Val::Px(70.);

const BUTTON_INFO: [(&str, MainMenuButton); 3] = [
    ("Play", MainMenuButton::Play),
    ("Setting", MainMenuButton::Setting),
    ("Quit", MainMenuButton::Quit),
];

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MenuState>()
            .add_systems(Update, menu_button_system.run_if(in_state(MenuState::Main)))
            .add_systems(OnEnter(MenuState::Main), main_menu_setup)
            .add_systems(OnExit(MenuState::Main), despawn_screen::<MainMenuScreen>)
            .add_systems(OnEnter(MenuState::Setting), settin_menu_setup)
            .add_systems(OnExit(MenuState::Setting), despawn_screen::<SettingScreen>);
    }
}

#[derive(Component)]
struct MainMenuScreen;

#[derive(Component)]
struct SettingScreen;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum MenuState {
    #[default]
    Main,
    Setting,
    Disabled,
}

#[derive(Component)]
enum MainMenuButton {
    Play,
    Setting,
    Quit,
}
#[derive(Component)]

enum _SettingButton {}

fn main_menu_setup(mut commands: Commands, font: Res<FontResource>) {
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
            MainMenuScreen,
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
                    parent.spawn(TextBundle::from_section("Invader Game", text_style(100.)));

                    for (text, action) in BUTTON_INFO {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    ..default()
                                },
                                action,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(text, text_style(60.)));
                            });
                    }
                });
        });
}

fn menu_button_system(
    interaction_query: Query<(&Interaction, &MainMenuButton), (Changed<Interaction>, With<Button>)>,
    mut main_state: ResMut<NextState<MainState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut app_exit_event: EventWriter<AppExit>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                MainMenuButton::Play => {
                    menu_state.set(MenuState::Disabled);
                    main_state.set(MainState::Game);
                }
                MainMenuButton::Setting => menu_state.set(MenuState::Setting),
                MainMenuButton::Quit => app_exit_event.send(AppExit),
            }
        }
    }
}

fn settin_menu_setup(mut commands: Commands, font: Res<FontResource>) {
    let _button_style = Style {
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
            SettingScreen,
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
                    parent.spawn(TextBundle::from_section("Setting", text_style(100.)));
                });
        });
}
