use std::time::Duration;

use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;

use super::enemy::{Enemy, EnemyAttack, EnemyCollider};
use crate::{game::*, WINDOW_WIDTH};
use crate::{Audio, MainState, SoundEvent, Texture, TextureResource};

pub struct PlayerPlugin<P: Clone, A: Clone, E: Clone> {
    pub setting: PluginSetting<P, A, E>,
}

impl<P: Clone, A: Clone + PartialEq, E: Clone> Plugin for PlayerPlugin<P, A, E>
where
    P: Component + PlayerMethod,
    A: Component + AttackMethod,
    E: Event + DamageEventMethod,
{
    fn build(&self, app: &mut App) {
        app.add_event::<E>()
            .add_systems(
                Update,
                (
                    move_player_attack::<A>,
                    player_damage_event::<P, E>,
                    player_collision::<P, E>,
                    attack_change::<P>,
                )
                    .run_if(in_state(self.setting.in_state)),
            )
            .add_systems(
                FixedUpdate,
                player_attack::<P, A>.run_if(in_state(self.setting.in_state)),
            );
    }
}

pub trait PlayerMethod
where
    Self: Default,
{
    fn hp(&self) -> isize;
    fn get_kill(&self) -> usize;
    fn damage(&mut self, damage: isize);
    fn kill(&mut self);
    fn get_attack(&self) -> AttackType;
    fn change_attack(&mut self, attack_type: AttackType);
}

pub trait AttackMethod {
    fn new(attack_type: AttackType) -> Self;
    fn attack(&self) -> AttackType;
    fn hp(&self) -> isize;
    fn damage(&mut self, damage: isize);
}

pub trait DamageEventMethod {
    fn power(&self) -> isize;
    fn event(attack: AttackType) -> Self;
}

#[derive(Component, Clone)]
pub struct Player {
    pub hp: isize,
    pub kill_count: usize,
    pub attack_type: AttackType,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            hp: INITIAL_PLAYER_HP,
            kill_count: INITIAL_KILLCOUNT,
            attack_type: AttackType::Normal,
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle<P: Component> {
    sprite_bundle: SpriteBundle,
    player: P,
}

#[derive(Bundle)]
pub struct PlayerAttackBundle<A: Component> {
    sprite_bundle: SpriteBundle,
    attack: A,
}

impl<P: Component + PlayerMethod> PlayerBundle<P> {
    pub fn new(player: P, texture: Texture, translation: Vec2) -> PlayerBundle<P> {
        PlayerBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: translation.extend(0.),
                    scale: PLAYER_SIZE.extend(0.0),
                    ..default()
                },
                texture,
                sprite: Sprite {
                    custom_size: Some(Vec2::new(2., 2.)),
                    ..default()
                },
                ..default()
            },
            player,
        }
    }
}

impl<A: Component> PlayerAttackBundle<A> {
    #[inline]
    pub fn new(attack: A, texture: Texture, trans: Vec3) -> PlayerAttackBundle<A> {
        PlayerAttackBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(trans.x, trans.y + PLAYER_SIZE.y / 2., 0.0),
                    scale: Vec3::new(20., 20., 0.),
                    ..default()
                },
                sprite: Sprite {
                    custom_size: Some(Vec2::new(2., 2.)),
                    ..default()
                },
                texture,
                ..default()
            },
            attack,
        }
    }
}

// プレイヤーがダメージを受けた時
#[inline]
fn player_damage_event<P, E>(
    mut player_query: Query<&mut P>,
    mut damage_event: EventReader<E>,
    mut update_info_event: EventWriter<UpdateInfo>,
    mut main_state: ResMut<NextState<MainState>>,
    mut game_mode: ResMut<NextState<GameMode>>,
) where
    P: Component + PlayerMethod,
    E: Event + DamageEventMethod,
{
    for event in damage_event.read() {
        let mut player = player_query.single_mut();

        let power = event.power();

        player.damage(power);

        if player.hp() <= 0 {
            game_mode.set(GameMode::Disabled);
            main_state.set(MainState::GameOver);
        }

        update_info_event.send_default();
    }
}

// プレイヤーのダメージ判定
#[inline]
fn player_collision<P, E>(
    mut commands: Commands,
    player_query: Query<&Transform, With<P>>,
    collider_query: Query<
        (Entity, &Transform, Option<&Enemy>, Option<&EnemyAttack>),
        With<EnemyCollider>,
    >,
    mut damage_event: EventWriter<E>,
    mut main_state: ResMut<NextState<MainState>>,
    mut game_mode: ResMut<NextState<GameMode>>,
) where
    P: Component + Default,
    E: Event + DamageEventMethod,
{
    let player_transform = player_query.single();

    let player_size = player_transform.scale.truncate();

    for (collider_entity, transform, enemy, attack) in &collider_query {
        let collision = collide(
            player_transform.translation,
            player_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if collision.is_some() {
            // 敵に衝突したなら
            if enemy.is_some() {
                game_mode.set(GameMode::Disabled);
                main_state.set(MainState::GameOver)
            }
            // 攻撃に衝突したなら
            if let Some(attack) = attack {
                commands.entity(collider_entity).despawn();

                damage_event.send(E::event(attack.0.attack.clone()));
            }
        }
    }
}

// プレイヤーを動かす
#[inline]
pub fn move_player<P: Component>(
    mut player_query: Query<&mut Transform, With<P>>,
    key: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut player_transform = player_query.single_mut();
    // 方向
    let mut direction_x = 0.0;
    let mut direction_y = 0.0;

    if key.pressed(KeyCode::Up) {
        direction_y += 1.0
    }
    if key.pressed(KeyCode::Down) {
        direction_y -= 1.0;
    }
    if key.pressed(KeyCode::Right) {
        direction_x += 1.0
    }
    if key.pressed(KeyCode::Left) {
        direction_x -= 1.0;
    }

    let new_player_position_x =
        player_transform.translation.x + direction_x * PLAYER_SPEED * time.delta_seconds();
    let new_player_position_y =
        player_transform.translation.y + direction_y * PLAYER_SPEED * time.delta_seconds();

    player_transform.translation.x = new_player_position_x.clamp(-CLAMP_X, CLAMP_X);
    player_transform.translation.y = new_player_position_y.clamp(-CLAMP_Y, CLAMP_Y);
}

// プレイヤーの攻撃
#[inline]
pub fn player_attack<P, A>(
    mut commands: Commands,
    player_query: Query<(&Transform, &P)>,
    texture: Res<TextureResource>,
    _key: Res<Input<KeyCode>>,
    mut sound_event: EventWriter<SoundEvent>,
) where
    P: Component + PlayerMethod,
    A: Component + AttackMethod,
{
    let (transform, player) = player_query.single();

    for attack_type in player.get_attack().list() {
        commands.spawn(PlayerAttackBundle::new(
            A::new(attack_type),
            texture.player_attack.clone(),
            transform.translation,
        ));
    }

    sound_event.send(SoundEvent(Audio::PlayerAttack));
}

// プレイヤーの攻撃を動かす
#[inline]
pub fn move_player_attack<A>(
    mut commands: Commands,
    mut attack_query: Query<(Entity, &mut Transform, &mut A), With<A>>,
) where
    A: Component + AttackMethod + Clone + PartialEq,
{
    for (entity, mut transform, mut attack) in &mut attack_query {
        let attack_type = attack.attack();

        transform.translation.y += attack_type.y_speed();
        transform.translation.x += attack_type.x_speed();

        let translation = transform.translation;

        if attack_type == AttackType::Rebound(true) && translation.x > WINDOW_WIDTH / 2. {
            *attack = A::new(AttackType::Rebound(false));
        } else if attack_type == AttackType::Rebound(false) && -WINDOW_WIDTH / 2. > translation.x {
            *attack = A::new(AttackType::Rebound(true));
        }

        if PLAYER_ATTACK_DESPAWN_POINT < transform.translation.y {
            commands.entity(entity).despawn()
        }
    }
}

pub const ATTACK_LIST: [AttackType; 4] = [
    AttackType::Rebound(false),
    AttackType::Shotgun,
    AttackType::Normal,
    AttackType::Power,
];
const ATTACK_LIST_LEN: usize = ATTACK_LIST.len();

static mut NEXT_ATTACK: usize = 1;

#[inline]
fn attack_change<P: Component + PlayerMethod>(
    mut player_query: Query<&mut P, With<P>>,
    key: Res<Input<KeyCode>>,
    mut time: ResMut<Time<Fixed>>,
) {
    let mut player = player_query.single_mut();

    if key.just_pressed(KeyCode::Space) {
        unsafe {
            let attack_type = ATTACK_LIST[NEXT_ATTACK];
            match attack_type {
                AttackType::Normal => time.set_timestep(Duration::from_secs_f32(0.08)),
                AttackType::Power => time.set_timestep(Duration::from_secs_f32(0.1)),
                _ => time.set_timestep(Duration::from_secs_f32(0.13)),
            }
            player.change_attack(attack_type);
            NEXT_ATTACK += 1;
            if NEXT_ATTACK == ATTACK_LIST_LEN {
                NEXT_ATTACK = 0;
            }
        }
    }
}
