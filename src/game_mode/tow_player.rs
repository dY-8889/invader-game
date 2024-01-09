use std::time::Duration;

use bevy::prelude::*;

use super::TEXT_PADDING;
use crate::entity::*;
use crate::game::*;
use crate::method_impl;
use crate::{FontResource, TextureResource};

const INITIAL_PLAYER1_POSITION: Vec2 = Vec2::new(100., -450.);
const INITIAL_PLAYER2_POSITION: Vec2 = Vec2::new(-100., -450.);

const INITIAL_PLAYER1_HP: isize = 30;
const INITIAL_PLAYER2_HP: isize = 30;

const TEXT_COLOR: Color = Color::WHITE;

pub struct TwoPlay;

impl Plugin for TwoPlay {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameMode::Tow), towplay_game_setup)
            .add_systems(
                Update,
                (
                    update_info,
                    move_player::<Player1>,
                    move_player_2,
                    enemy_collision::<Player2, PlayerAttack2>,
                )
                    .run_if(in_state(GameMode::Tow)),
            )
            .add_plugins(GamePlayPlugin::<Player1, PlayerAttack1, Player1Damage> {
                setting: PluginSetting {
                    enemy_create_timer: Duration::from_secs_f32(2.5),
                    enemy_attack_timer: Duration::from_secs_f32(1.0),
                    in_state: GameMode::Tow,
                    ..default()
                },
            })
            .add_plugins(PlayerPlugin::<Player2, PlayerAttack2, Player2Damage> {
                setting: PluginSetting {
                    in_state: GameMode::Tow,
                    ..default()
                },
            });
    }
}

#[derive(Event, Clone)]
pub struct Player1Damage {
    attack: AttackType,
}

#[derive(Event, Clone)]
pub struct Player2Damage {
    attack: AttackType,
}

#[derive(Component, Clone, PartialEq)]
pub struct PlayerAttack1(Attack);

#[derive(Component, Clone, Copy, PartialEq)]
pub struct PlayerAttack2(Attack);

#[derive(Component, Default, Clone)]
pub struct Player1(Player);

#[derive(Component, Default, Clone)]
pub struct Player2(Player);

#[derive(Component)]
enum BoardSection {
    Hp1,
    Kill1,
    Hp2,
    Kill2,
}

fn towplay_game_setup(
    mut commands: Commands,
    texture: Res<TextureResource>,
    font: Res<FontResource>,
) {
    let text_style = TextStyle {
        font: font.0.clone(),
        font_size: 70.,
        color: TEXT_COLOR,
    };
    let name_style = TextStyle {
        font: font.0.clone(),
        font_size: 80.,
        color: TEXT_COLOR,
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    position_type: PositionType::Absolute,
                    top: TEXT_PADDING,
                    left: TEXT_PADDING,
                    ..default()
                },
                ..default()
            },
            PlayerInfoScreen,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Player1", name_style.clone()));
            parent.spawn((
                TextBundle::from_sections([
                    TextSection::new("HP", text_style.clone()),
                    TextSection::new(INITIAL_PLAYER1_HP.to_string(), text_style.clone()),
                ]),
                BoardSection::Hp1,
            ));
            parent.spawn((
                TextBundle::from_sections([
                    TextSection::new("KILL", text_style.clone()),
                    TextSection::new(INITIAL_KILLCOUNT.to_string(), text_style.clone()),
                ]),
                BoardSection::Kill1,
            ));
        });
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::Start,
                    justify_self: JustifySelf::End,
                    top: TEXT_PADDING,
                    right: TEXT_PADDING,
                    ..default()
                },
                ..default()
            },
            PlayerInfoScreen,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Player2", name_style));
            parent.spawn((
                TextBundle::from_sections([
                    TextSection::new("HP", text_style.clone()),
                    TextSection::new(INITIAL_PLAYER2_HP.to_string(), text_style.clone()),
                ]),
                BoardSection::Hp2,
            ));
            parent.spawn((
                TextBundle::from_sections([
                    TextSection::new("KILL", text_style.clone()),
                    TextSection::new(INITIAL_KILLCOUNT.to_string(), text_style.clone()),
                ]),
                BoardSection::Kill2,
            ));
        });

    commands.spawn(PlayerBundle::new(
        Player1(Player {
            hp: INITIAL_PLAYER1_HP,
            kill_count: 0,
            attack_type: AttackType::Normal,
        }),
        texture.player.clone(),
        INITIAL_PLAYER1_POSITION,
    ));
    commands.spawn(PlayerBundle::new(
        Player2(Player {
            hp: INITIAL_PLAYER2_HP,
            kill_count: 0,
            attack_type: AttackType::Normal,
        }),
        texture.player.clone(),
        INITIAL_PLAYER2_POSITION,
    ));
}

fn move_player_2(
    mut player_query: Query<&mut Transform, With<Player2>>,
    key: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    println!("{}", time.delta_seconds());
    let mut player_transform = player_query.single_mut();
    // 方向
    let mut direction_x = 0.0;
    let mut direction_y = 0.0;

    if key.pressed(KeyCode::W) {
        direction_y += 1.0
    }
    if key.pressed(KeyCode::S) {
        direction_y -= 1.0;
    }
    if key.pressed(KeyCode::D) {
        direction_x += 1.0
    }
    if key.pressed(KeyCode::A) {
        direction_x -= 1.0;
    }

    let new_player_position_x =
        player_transform.translation.x + direction_x * PLAYER_SPEED * time.delta_seconds();
    let new_player_position_y =
        player_transform.translation.y + direction_y * PLAYER_SPEED * time.delta_seconds();

    player_transform.translation.x = new_player_position_x.clamp(-CLAMP_X, CLAMP_X);
    player_transform.translation.y = new_player_position_y.clamp(-CLAMP_Y, CLAMP_Y);
}

fn update_info(
    player1_query: Query<&Player1>,
    player2_query: Query<&Player2>,
    mut text_query: Query<(&mut Text, &BoardSection)>,
    mut update_info_event: EventReader<UpdateInfo>,
) {
    if !update_info_event.is_empty() {
        update_info_event.clear();

        let player1 = player1_query.single();
        let player2 = player2_query.single();

        for (mut text, section) in &mut text_query {
            match section {
                BoardSection::Hp1 => text.sections[1].value = player1.hp().to_string(),
                BoardSection::Kill1 => text.sections[1].value = player1.get_kill().to_string(),
                BoardSection::Hp2 => text.sections[1].value = player2.hp().to_string(),
                BoardSection::Kill2 => text.sections[1].value = player2.get_kill().to_string(),
            }
        }
    }
}

method_impl!(Player1, PlayerAttack1, Player1Damage);
method_impl!(Player2, PlayerAttack2, Player2Damage);
