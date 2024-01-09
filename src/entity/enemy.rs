use bevy::prelude::*;

use bevy::sprite::collide_aabb::collide;
use bevy::time::common_conditions::on_timer;

use rand::{seq::SliceRandom, thread_rng, Rng};

use super::AttackMethod;
use super::PlayerMethod;
use crate::game::*;
use crate::{Texture, TextureResource, WINDOW_HEIGHT, WINDOW_WIDTH};

pub struct EnemyPlugin<P: Clone, A: Clone, E: Clone> {
    pub setting: PluginSetting<P, A, E>,
}

impl<P: Clone, A: Clone, E: Clone> Plugin for EnemyPlugin<P, A, E>
where
    P: Component + PlayerMethod,
    A: Component + AttackMethod,
    E: Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        let state = self.setting.in_state;
        app.add_systems(OnExit(state), entity_despawn::<P, A>)
            .add_systems(
                Update,
                (
                    move_enemy,
                    move_enemy_attack,
                    enemy_attack.run_if(on_timer(self.setting.enemy_attack_timer)),
                    create_enemy.run_if(on_timer(self.setting.enemy_create_timer)),
                    enemy_collision::<P, A>,
                )
                    .run_if(in_state(state)),
            );
    }
}

#[derive(Component)]
pub struct EnemyAttack(pub Attack);

#[derive(Component)]
pub struct EnemyCollider;

#[derive(Component, Clone, Copy)]
pub enum EnemyType {
    Normal,
    Drop,
}

#[derive(Component, Clone, Copy)]
pub struct Enemy {
    pub hp: isize,
    pub enemy_type: EnemyType,
}

#[derive(Bundle)]
pub struct EnemyBundle {
    sprite_bundle: SpriteBundle,
    enemy: Enemy,
    collider: EnemyCollider,
}

#[derive(Bundle)]
pub struct EnemyAttackBundle {
    sprite_bundle: SpriteBundle,
    attack: EnemyAttack,
    collider: EnemyCollider,
}

impl Enemy {
    #[inline]
    fn damage(&mut self, damage: isize) {
        self.hp -= damage
    }
}

impl EnemyType {
    const ENEMY_TYPES: [EnemyType; 2] = [EnemyType::Normal, EnemyType::Drop];

    #[inline]
    const fn hp(&self) -> isize {
        match self {
            EnemyType::Normal => 25,
            EnemyType::Drop => 30,
        }
    }
    #[inline]
    pub const fn speed(&self) -> f32 {
        match self {
            EnemyType::Normal => 0.0,
            EnemyType::Drop => 250.0,
        }
    }
    #[inline]
    const fn scale(&self) -> Vec2 {
        match self {
            EnemyType::Normal => Vec2::new(50., 40.),
            EnemyType::Drop => Vec2::new(30., 40.),
        }
    }
    #[inline]
    fn translation(&self) -> Vec2 {
        let range = match self {
            EnemyType::Normal => (
                -WINDOW_WIDTH as i32 / 2..WINDOW_WIDTH as i32 / 2,
                0..WINDOW_HEIGHT as i32 / 2,
            ),
            EnemyType::Drop => (
                -WINDOW_WIDTH as i32 / 2..WINDOW_WIDTH as i32 / 2,
                WINDOW_HEIGHT as i32 / 4..WINDOW_HEIGHT as i32,
            ),
        };

        let mut rng = thread_rng();

        let x = rng.gen_range(range.0);
        let y = rng.gen_range(range.1);

        Vec2::new(x as f32, y as f32)
    }
    #[inline]
    pub fn random() -> EnemyType {
        *Self::ENEMY_TYPES.choose(&mut thread_rng()).unwrap()
    }
}

impl EnemyBundle {
    #[inline]
    pub fn new(enemy_type: EnemyType, texture: Texture) -> EnemyBundle {
        EnemyBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: enemy_type.translation().extend(0.0),
                    scale: enemy_type.scale().extend(0.0),
                    ..default()
                },
                sprite: Sprite {
                    custom_size: Some(Vec2::new(2., 2.)),
                    ..default()
                },
                texture,
                ..default()
            },
            enemy: Enemy {
                hp: enemy_type.hp(),
                enemy_type,
            },
            collider: EnemyCollider,
        }
    }
}

impl EnemyAttackBundle {
    #[inline]
    pub fn new(attack: AttackType, texture: Texture, translation: Vec3) -> EnemyAttackBundle {
        EnemyAttackBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation,
                    scale: attack.scale().extend(0.0),
                    ..default()
                },
                sprite: Sprite {
                    custom_size: Some(Vec2::new(2., 2.)),
                    ..default()
                },
                texture,
                ..default()
            },
            attack: EnemyAttack(Attack::new(attack)),
            collider: EnemyCollider,
        }
    }
}

// 敵のダメージ判定
#[inline]
pub fn enemy_collision<T, A>(
    mut commands: Commands,
    mut player_query: Query<&mut T>,
    mut enemy_query: Query<
        (
            Entity,
            &Transform,
            Option<&mut Enemy>,
            Option<&mut EnemyAttack>,
        ),
        With<EnemyCollider>,
    >,
    mut attack_query: Query<(Entity, &Transform, &mut A), With<A>>,
    mut update_info_event: EventWriter<UpdateInfo>,
) where
    T: Component + PlayerMethod,
    A: Component + AttackMethod,
{
    for (player_attack_entity, transform, mut player_attack) in &mut attack_query {
        for (enemy_entity, enemy_transform, mut enemy, enemy_attack) in &mut enemy_query {
            let e_translation = enemy_transform.translation;
            let e_scale = enemy_transform.scale.truncate();

            let collision = collide(
                e_translation,
                e_scale,
                transform.translation,
                transform.scale.truncate(),
            );
            if collision.is_some() {
                let mut player = player_query.single_mut();

                if let Some(enemy) = enemy.as_mut() {
                    enemy.damage(player_attack.hp());

                    if enemy.hp <= 0 {
                        player.kill();

                        update_info_event.send_default();

                        commands.entity(enemy_entity).despawn();
                    } else {
                        player_attack.damage(enemy.hp);
                        commands.entity(player_attack_entity).despawn();
                    }
                }
                if let Some(mut enemy_attack) = enemy_attack {
                    let enemy_attack_hp = enemy_attack.0.hp;
                    let player_attack_hp = player_attack.hp();

                    if enemy_attack_hp < player_attack_hp {
                        player_attack.damage(enemy_attack_hp);

                        commands.entity(enemy_entity).despawn();
                    } else if player_attack_hp < enemy_attack_hp {
                        enemy_attack.0.damage(player_attack_hp);

                        commands.entity(player_attack_entity).despawn();
                    } else {
                        commands.entity(enemy_entity).despawn();
                        commands.entity(player_attack_entity).despawn();
                    }
                }
            }
        }
    }
}

// 敵を動かす
#[inline]
fn move_enemy(
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &mut Transform, &Enemy)>,
    time: Res<Time>,
) {
    for (entity, mut transform, enemy) in &mut enemy_query {
        let enemy_type = enemy.enemy_type;
        transform.translation.y -= enemy_type.speed() * time.delta_seconds();

        if transform.translation.y <= -CLAMP_Y {
            commands.entity(entity).despawn();
        }
    }
}

// 敵の攻撃
#[inline]
fn enemy_attack(
    mut commands: Commands,
    texture: Res<TextureResource>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    for transform in &enemy_query {
        commands.spawn(EnemyAttackBundle::new(
            AttackType::Normal,
            texture.enemy_attack.clone(),
            transform.translation,
        ));
    }
}

// 敵の攻撃を動かす
#[inline]
pub fn move_enemy_attack(
    mut commands: Commands,
    mut attack_query: Query<(Entity, &mut Transform, &EnemyAttack), With<EnemyAttack>>,
) {
    for (entity, mut transform, attack) in &mut attack_query {
        let attack = attack.0.attack;

        transform.translation.y -= attack.y_speed();
        transform.translation.x -= attack.x_speed();

        transform.translation.y -= ENEMY_ATTACK_SPEED / 2.;
        if ENEMY_ATTACK_DESPAWN_POINT > transform.translation.y {
            commands.entity(entity).despawn()
        }
    }
}

// 敵を作る
#[inline]
fn create_enemy(mut commands: Commands, texture: Res<TextureResource>) {
    for _ in 1..=ENEMY_CREAT_NUMBER {
        commands.spawn(EnemyBundle::new(EnemyType::random(), texture.enemy.clone()));
    }
}
