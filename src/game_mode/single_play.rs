use std::time::Duration;

use bevy::prelude::*;

use super::TEXT_PADDING;
use crate::entity::*;
use crate::game::*;
use crate::method_impl;
use crate::{FontResource, TextureResource};

pub struct SinglePlay;

impl Plugin for SinglePlay {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameMode::Single), single_game_setup)
            .add_systems(
                Update,
                (update_info::<SinglePlayer>, move_player::<SinglePlayer>)
                    .run_if(in_state(GameMode::Single)),
            )
            .add_plugins(
                GamePlayPlugin::<SinglePlayer, SinglePlayerAttack, SinglePlayerEvent> {
                    setting: PluginSetting {
                        player_attack_timer: Duration::from_secs_f32(0.18),
                        enemy_attack_timer: Duration::from_secs_f32(0.7),
                        enemy_create_timer: Duration::from_secs_f32(4.0),
                        in_state: GameMode::Single,
                        ..default()
                    },
                },
            );
    }
}

#[derive(Event, Clone)]
struct SinglePlayerEvent {
    attack: AttackType,
}

#[derive(Component, Clone, PartialEq)]
pub struct SinglePlayerAttack(Attack);

#[derive(Component, Default, Clone)]
pub struct SinglePlayer(Player);

#[derive(Component)]
enum BoardSection {
    Hp,
    Kill,
}

fn single_game_setup(
    mut commands: Commands,
    texture: Res<TextureResource>,
    font: Res<FontResource>,
) {
    let text_style = TextStyle {
        font: font.0.clone(),
        font_size: 70.,
        color: Color::WHITE,
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
            parent.spawn((
                TextBundle::from_sections([
                    TextSection::new("HP: ", text_style.clone()),
                    TextSection::new(INITIAL_PLAYER_HP.to_string(), text_style.clone()),
                ]),
                BoardSection::Hp,
            ));
            parent.spawn((
                TextBundle::from_sections([
                    TextSection::new("Kill: ", text_style.clone()),
                    TextSection::new(INITIAL_KILLCOUNT.to_string(), text_style),
                ]),
                BoardSection::Kill,
            ));
        });

    commands.spawn(PlayerBundle::new(
        SinglePlayer(Player {
            hp: 100,
            attack_type: ATTACK_LIST[0],
            ..default()
        }),
        texture.player.clone(),
        INITIAL_PLAYER_POSITION,
    ));
}

fn update_info<P: Component + PlayerMethod>(
    player_query: Query<&P>,
    mut text_query: Query<(&mut Text, &BoardSection)>,
    mut update_info_event: EventReader<UpdateInfo>,
) {
    if !update_info_event.is_empty() {
        update_info_event.clear();

        let player = player_query.single();
        for (mut text, section) in &mut text_query {
            match section {
                BoardSection::Hp => text.sections[1].value = player.hp().to_string(),
                BoardSection::Kill => text.sections[1].value = player.get_kill().to_string(),
            }
        }
    }
}

method_impl!(SinglePlayer, SinglePlayerAttack, SinglePlayerEvent);
