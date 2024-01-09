use std::net::UdpSocket;
use std::str;
use std::time::Duration;

use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy::time::common_conditions::on_timer;

use bevy_simple_text_input::{TextInput, TextInputSubmitEvent};
use local_ip_address::local_ip;

use crate::entity::PlayerAttackBundle;
use crate::{game::*, FontResource};
use crate::{Audio, SoundEvent, TextureResource};

const INITIAL_OPPONENT_POSITION: Vec2 = Vec2::new(0., 350.);
const PLAYER_ATTACK_SPEED: f32 = 15.0;

const READ_TIMEOUT: Option<Duration> = Some(Duration::from_millis(25));
const UPDATE_TIMER: Duration = Duration::from_millis(25);
const PLAYER_SEND_TIMER: Duration = Duration::from_millis(25);

const MY_SPEED: f32 = 400.;

pub struct VSPlayer;

impl Plugin for VSPlayer {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameMode::VS), vs_player_setup)
            .add_systems(
                Update,
                (
                    // 自分
                    player_collision,
                    move_my_player,
                    move_player_attack,
                    player_attack,
                    position_send.run_if(on_timer(PLAYER_SEND_TIMER)),
                    // 敵
                    move_opponent.run_if(on_timer(UPDATE_TIMER)),
                    move_opponent_attack,
                    //
                    address_change,
                    focus,
                )
                    .run_if(in_state(GameMode::VS)),
            );
    }
}

#[derive(Resource)]
struct Server(UdpSocket);

#[derive(Component)]
struct My;

#[derive(Component)]
struct Opponent;

#[derive(Component)]
struct MyAttack;

#[derive(Component)]
struct OpponentAttack;

fn vs_player_setup(
    mut commands: Commands,
    texture: Res<TextureResource>,
    font: Res<FontResource>,
    assets_font: Res<Assets<Font>>,
) {
    for font in assets_font.iter() {
        println!("{:?}", font);
    }
    let ip = local_ip().unwrap().to_string();

    let server = UdpSocket::bind(ip + ":8080").expect("サーバーエラー");
    server.connect("".to_string()).expect("connect エラー");

    server.set_read_timeout(READ_TIMEOUT).unwrap();

    commands.insert_resource(Server(server));

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: INITIAL_PLAYER_POSITION.extend(0.),
                scale: PLAYER_SIZE.extend(0.0),
                ..default()
            },
            texture: texture.player.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(2., 2.)),
                ..default()
            },
            ..default()
        },
        My,
    ));
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: INITIAL_OPPONENT_POSITION.extend(0.),
                scale: PLAYER_SIZE.extend(0.0),
                ..default()
            },
            texture: texture.player.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(2., 2.)),
                flip_y: true,
                ..default()
            },
            ..default()
        },
        Opponent,
    ));

    commands
        .spawn(NodeBundle {
            style: Style {
                top: Val::Px(7.),
                left: Val::Px(7.),
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(300.0),
                        height: Val::Px(50.),
                        border: UiRect::all(Val::Px(5.0)),
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    border_color: Color::WHITE.into(),
                    background_color: Color::WHITE.into(),
                    ..default()
                },
                TextInput {
                    text_style: TextStyle {
                        font: font.0.clone(),
                        font_size: 40.,
                        color: Color::BLACK,
                    },
                    inactive: true,
                },
            ));
        });
}

const TEXT_INPUT_ACTIVE: Color = Color::YELLOW;
const TEXT_INPUT_INACTIVE: Color = Color::WHITE;

fn address_change(server: Res<Server>, mut event: EventReader<TextInputSubmitEvent>) {
    for event in event.read() {
        let addr = event.value.clone();
        if let Err(e) = server.0.connect(addr.clone()) {
            println!("connect err addr: {} {}", addr, e)
        } else {
            println!("connect Ok: {}", addr)
        }
    }
}

fn focus(
    mut text_input_query: Query<(&mut TextInput, &mut BorderColor)>,
    key: Res<Input<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Escape) {
        for (mut text_input, mut border_color) in &mut text_input_query {
            if text_input.inactive {
                text_input.inactive = false;
                *border_color = TEXT_INPUT_ACTIVE.into();
            } else {
                text_input.inactive = true;
                *border_color = TEXT_INPUT_INACTIVE.into();
            }
        }
    }
}

// プレイヤーに攻撃が当たると消える
fn player_collision(
    mut commands: Commands,
    player_query: Query<&Transform, With<My>>,
    collider_query: Query<(Entity, &Transform), With<OpponentAttack>>,
) {
    let player_transform = player_query.single();

    let player_size = player_transform.scale.truncate();

    for (collider_entity, transfrom) in &collider_query {
        let collision = collide(
            player_transform.translation,
            player_size,
            transfrom.translation,
            transfrom.scale.truncate(),
        );
        if collision.is_some() {
            commands.entity(collider_entity).despawn();
        }
    }
}

// プレイヤーを動かす
fn move_my_player(
    mut my_query: Query<&mut Transform, With<My>>,
    time: Res<Time>,
    key: Res<Input<KeyCode>>,
) {
    let mut transform = my_query.single_mut();

    let mut direction_x: f32 = 0.0;
    let mut direction_y: f32 = 0.0;

    if key.pressed(KeyCode::Up) {
        direction_y += 1.0
    }
    if key.pressed(KeyCode::Down) {
        direction_y += -1.0
    }
    if key.pressed(KeyCode::Right) {
        direction_x += 1.0
    }
    if key.pressed(KeyCode::Left) {
        direction_x += -1.0
    }

    let new_position_x = transform.translation.x + direction_x * MY_SPEED * time.delta_seconds();
    let new_position_y = transform.translation.y + direction_y * MY_SPEED * time.delta_seconds();

    transform.translation.x = new_position_x.clamp(-CLAMP_X, CLAMP_X);
    transform.translation.y = new_position_y.clamp(-CLAMP_Y, CLAMP_Y);
}

// 敵を動かす
#[inline]
fn move_opponent(
    mut commands: Commands,
    mut query: Query<&mut Transform, With<Opponent>>,
    server: Res<Server>,
    texture: Res<TextureResource>,
) {
    let mut buf = [0; 2048];

    let Ok(buf_size) = server.0.recv(&mut buf) else {
        println!("recv timeout");
        return;
    };

    let buf = &buf[..buf_size];
    let str = str::from_utf8(buf).unwrap();

    let mut vec: Vec<&str> = str.split_ascii_whitespace().collect();
    let pop = vec.pop().unwrap();

    let mut transform = query.single_mut();

    match pop {
        "p" => transform.translation = to_pos(vec),
        // 敵の攻撃
        "a" => {
            commands.spawn(PlayerAttackBundle::new(
                OpponentAttack,
                texture.player_attack.clone(),
                transform.translation,
            ));
        }
        _ => (),
    }
}

#[inline]
fn to_pos(str: Vec<&str>) -> Vec3 {
    let x: f32 = str[0].trim().parse().unwrap();
    let y: f32 = str[1].trim().parse().unwrap();
    Vec3::new(-x, -y, 0.0)
}

fn position_send(player_query: Query<&Transform, With<My>>, server: Res<Server>) {
    let pos = player_query.single().translation;

    let pos = format!("{} {} p", pos.x, pos.y);
    if let Err(e) = server.0.send(pos.as_bytes()) {
        println!("send e: {}", e);
    }
}

fn player_attack(
    mut commands: Commands,
    player_query: Query<&Transform, With<My>>,
    texture: Res<TextureResource>,
    key: Res<Input<KeyCode>>,
    server: Res<Server>,
    mut sound_event: EventWriter<SoundEvent>,
) {
    if key.just_pressed(KeyCode::Space) {
        if let Err(e) = server.0.send("a".as_bytes()) {
            println!("{:?}", e);
        }

        let transform = player_query.single();

        commands.spawn(PlayerAttackBundle::new(
            MyAttack,
            texture.player_attack.clone(),
            transform.translation,
        ));

        sound_event.send(SoundEvent(Audio::PlayerAttack));
    }
}

// プレイヤーの攻撃を動かす
fn move_player_attack(
    mut commands: Commands,
    mut attack_query: Query<(Entity, &mut Transform), With<MyAttack>>,
) {
    for (entity, mut transform) in &mut attack_query {
        transform.translation.y += PLAYER_ATTACK_SPEED;
        if PLAYER_ATTACK_DESPAWN_POINT < transform.translation.y {
            commands.entity(entity).despawn()
        }
    }
}

// 敵の攻撃を動かす
fn move_opponent_attack(
    mut commands: Commands,
    mut attack_query: Query<(Entity, &mut Transform), With<OpponentAttack>>,
) {
    for (entity, mut transform) in &mut attack_query {
        transform.translation.y -= PLAYER_ATTACK_SPEED;
        if -PLAYER_ATTACK_DESPAWN_POINT > transform.translation.y {
            commands.entity(entity).despawn()
        }
    }
}
