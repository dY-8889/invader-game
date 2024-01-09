use bevy::{audio::VolumeLevel, prelude::*};
use bevy_simple_text_input::TextInputPlugin;

mod entity;
mod game;
mod game_mode;
mod menu;

pub const WINDOW_WIDTH: f32 = 700.0;
pub const WINDOW_HEIGHT: f32 = 1050.0;

const BACKGROUND_DESPAWN_POINT: f32 = -WINDOW_HEIGHT + 40.;
const BACKGROUND_SPAWN_POINT: f32 = WINDOW_HEIGHT;
const BACKGROUND_SPEED: f32 = 1.0;

type Sound = Handle<AudioSource>;
pub type Texture = Handle<Image>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum MainState {
    #[default]
    Menu,
    Game,
    GameOver,
}

#[derive(Component)]
struct BackgroundScreen;

#[derive(Resource)]
pub struct TextureResource {
    pub player_attack: Texture,
    pub player: Texture,
    pub enemy: Texture,
    pub enemy_attack: Texture,
}

#[derive(Resource)]
pub struct SoundResource {
    attack: Sound,
}

#[derive(Resource)]
pub struct FontResource(pub Handle<Font>);

#[derive(Event)]
pub struct SoundEvent(Audio);

#[derive(Clone, Copy)]
pub enum Audio {
    PlayerAttack,
}

impl SoundResource {
    #[inline]
    fn get(&self, audio: Audio) -> &Sound {
        match audio {
            Audio::PlayerAttack => &self.attack,
        }
    }
}

fn main() {
    App::new()
        .add_state::<MainState>()
        .add_event::<SoundEvent>()
        .init_resource::<Assets<Font>>()
        .add_systems(Startup, setup)
        .add_systems(Update, (sound_event, background_move))
        .add_plugins((
            menu::MenuPlugin,
            game::GamePlugin,
            game::GameOver,
            TextInputPlugin,
        ))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..default()
                },
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .run();
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(TextureResource {
        player_attack: assets_server.load("images/player_attack.png"),
        player: assets_server.load("images/player.png"),
        enemy: assets_server.load("images/enemy.png"),
        enemy_attack: assets_server.load("images/enemy_attack.png"),
    });

    commands.insert_resource(SoundResource {
        attack: assets_server.load("audio/player_attack.ogg"),
    });

    commands.insert_resource(FontResource(
        assets_server.load("font/SairaSemiCondensed-Black.ttf"),
    ));

    let background = assets_server.load("images/background.png");

    commands.spawn((
        SpriteBundle {
            texture: background.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, -1.0),
                ..default()
            },
            ..default()
        },
        BackgroundScreen,
    ));
    commands.spawn((
        SpriteBundle {
            texture: background,
            transform: Transform {
                translation: Vec3::new(0.0, BACKGROUND_SPAWN_POINT, -1.0),
                ..default()
            },
            ..default()
        },
        BackgroundScreen,
    ));
}

fn sound_event(
    mut commands: Commands,
    mut sound_event: EventReader<SoundEvent>,
    sound: Res<SoundResource>,
) {
    for event in sound_event.read() {
        commands.spawn(AudioBundle {
            source: sound.get(event.0).clone(),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: bevy::audio::Volume::Absolute(VolumeLevel::new(1.0)),
                ..default()
            },
        });
    }
}

fn background_move(mut background_query: Query<&mut Transform, With<BackgroundScreen>>) {
    for mut transform in &mut background_query {
        transform.translation.y -= BACKGROUND_SPEED;
        if transform.translation.y <= BACKGROUND_DESPAWN_POINT {
            transform.translation.y = BACKGROUND_SPAWN_POINT;
        }
    }
}

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive()
    }
}
