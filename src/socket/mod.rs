pub mod client;
pub mod connection;
pub mod message;
pub mod player;
pub mod refs;
pub mod request;
pub mod response;
pub mod room;

use self::client::{Client, SocketEvent, SocketStatus};
use self::player::Player;
use self::request::Request;
use self::response::Response;
use crate::player::FriendUpdateEvent;
use crate::schedule::UpdateSet;
use crate::socket::connection::{connect_socket, create_channel, get_socket_url};
use bevy::prelude::*;
use bevy::text::BreakLineOn;
use bevy::utils::HashMap;
use chrono::Utc;
use tokio::sync::mpsc::Receiver;

pub const GAME_ROOM: &str = "iso";
pub const HEARTBEAT_INTERVAL_SECS: f32 = 15.0;

#[derive(Clone, Debug)]
pub struct Config;

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Component)]
struct SocketInfo {
    text_section: TextSection,
}

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
pub struct HeartbeatTimer {
    timer: Timer,
}

impl Default for HeartbeatTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(HEARTBEAT_INTERVAL_SECS, TimerMode::Repeating),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SocketPlugin {
    pub config: Config,
}

impl Default for SocketPlugin {
    fn default() -> Self {
        Self {
            config: Config::default(),
        }
    }
}

impl Plugin for SocketPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameSocket::new())
            .insert_resource(HeartbeatTimer::default())
            .register_type::<HeartbeatTimer>()
            .add_systems(Startup, spawn_socket_info)
            .add_systems(
                Update,
                (handle_socket_events, update_socket_info)
                    .chain()
                    .in_set(UpdateSet::AfterEffects),
            )
            .add_systems(Update, send_heartbeat);
    }
}

fn send_heartbeat(
    mut heartbeat_timer: ResMut<HeartbeatTimer>,
    time: Res<Time>,
    socket: Res<GameSocket>,
) {
    heartbeat_timer.timer.tick(time.delta());
    if !heartbeat_timer.timer.just_finished() {
        return;
    }
    let request = Request::new_heartbeat();
    socket.handle.call(request).expect("heartbeat error");
}

fn spawn_socket_info(mut commands: Commands, asset_server: Res<AssetServer>) {
    let text_section = TextSection::new(
        "...",
        TextStyle {
            font: asset_server.load("fonts/FiraCode-Regular.otf"),
            font_size: 14.,
            color: Color::ANTIQUE_WHITE,
            ..default()
        },
    );

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Vw(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                ..default()
            },
            Name::new("SocketInfo"),
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text {
                        sections: vec![text_section.clone()],
                        justify: default(),
                        linebreak_behavior: BreakLineOn::WordBoundary,
                    },
                    ..default()
                },
                SocketInfo { text_section },
                Name::new("SocketInfoText"),
            ));
        });
}

fn update_socket_info(
    mut socket_info_query: Query<(&mut Text, &SocketInfo), With<SocketInfo>>,
    socket: Res<GameSocket>,
) {
    let Ok((mut text, socket_info)) = socket_info_query.get_single_mut() else {
        return;
    };

    let socket_status = match &socket.status {
        Some(s) => format!("{:?}", s),
        None => "None".to_string(),
    };

    let player = socket.get_player();

    let player_position = match player.position {
        Some(pos) => format!("({:.2}, {:.2})", pos.x, pos.z),
        None => "()".to_string(),
    };

    let friends_info: Vec<String> = socket
        .get_friends()
        .iter()
        .map(|(_uuid, player)| {
            let position = match player.position {
                Some(pos) => format!("({:.2}, {:.2})", pos.x, pos.z),
                None => "()".to_string(),
            };
            format!("@{} {}", player.username, position)
        })
        .collect();

    let info_text = format!(
        "{socket_status} @{} {} friends={:?}",
        player.username, player_position, friends_info
    );

    let mut text_section = socket_info.text_section.clone();
    text_section.value = info_text;
    text.sections = vec![text_section];
}

#[derive(Debug, Resource)]
pub struct GameSocket {
    pub player_uuid: String,
    pub players: HashMap<String, Player>,
    pub status: Option<SocketStatus>,
    pub last_response: Option<Response>,
    pub handle: ezsockets::Client<Client>,
    pub rx: Receiver<SocketEvent>,
    _runtime: tokio::runtime::Runtime,
}

impl GameSocket {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        debug!("create_channel");
        let (tx, rx) = create_channel();

        let (handle, future) = runtime.block_on(async move {
            let socket_url = get_socket_url();
            debug!("connect_socket={:?}", &socket_url);
            let (handle, future) = connect_socket(tx).await;
            (handle, future)
        });

        runtime.spawn(async move {
            future.await.unwrap();
        });

        let player = Player::new_with_username_from_env_or_generate();
        let mut players = HashMap::new();
        players.insert(player.uuid.clone(), player.clone());

        Self {
            handle,
            rx,
            status: None,
            last_response: None,
            player_uuid: player.uuid,
            players,
            _runtime: runtime,
        }
    }

    pub fn get_player(&self) -> &Player {
        self.players.get(&self.player_uuid).unwrap()
    }

    pub fn get_friend(&self, player_uuid: &String) -> Option<Player> {
        match self.get_friends().get(player_uuid) {
            Some(friend) => Some(friend.clone()),
            None => None,
        }
    }

    pub fn get_friends(&self) -> HashMap<String, Player> {
        let mut friends = self.players.clone();
        friends.remove(&self.player_uuid);
        friends
    }

    pub fn is_player_self(&self, player_uuid: &String) -> bool {
        &self.player_uuid == player_uuid
    }

    pub fn remove_friend(&mut self, player: Player) {
        if !self.is_player_self(&player.uuid) {
            self.players.remove(&player.uuid);
        }
    }

    pub fn upsert_player(&mut self, player: Player) {
        self.players
            .entry(player.uuid.clone())
            .and_modify(|existing_player| {
                if player.position.is_some() {
                    existing_player.position = player.position;
                }
            })
            .or_insert_with(|| player.clone());
    }

    pub fn upsert_players(&mut self, players: Vec<Player>) {
        for player in players {
            self.upsert_player(player);
        }
    }

    pub fn update_player_position(&mut self, player_uuid: String, position: Vec3) {
        if let Some(player) = self.players.get_mut(&player_uuid) {
            player.position = Some(position);
        }
    }

    pub fn has_spawned(&mut self, player_uuid: &String) -> bool {
        if let Some(player) = self.players.get(player_uuid) {
            player.spawned_at.is_some()
        } else {
            false
        }
    }

    pub fn set_spawned_at(&mut self, player_uuid: &String) {
        if let Some(player) = self.players.get_mut(player_uuid) {
            let now = Utc::now();
            let timestamp = now.timestamp() as u64;
            player.spawned_at = Some(timestamp);
        }
    }
}

fn handle_socket_events(
    mut socket: ResMut<GameSocket>,
    mut update_event_writer: EventWriter<FriendUpdateEvent>,
) {
    match socket.rx.try_recv() {
        Ok(socket_event) => match socket_event {
            SocketEvent::Close => socket.status = Some(SocketStatus::Closed),
            SocketEvent::Connect => {
                socket.status = Some(SocketStatus::Connected);
                let request = Request::new_join(GAME_ROOM.into(), socket.get_player().clone());
                socket.handle.call(request).expect("join error");
            }
            SocketEvent::ConnectFail => socket.status = Some(SocketStatus::ConnectFailed),
            SocketEvent::Disconnect => socket.status = Some(SocketStatus::Disconnected),
            SocketEvent::Response(response) => {
                socket.last_response = Some(response.clone());

                match response {
                    Response::PlayerUpdate(player_update) => {
                        socket.update_player_position(
                            player_update.player_uuid.clone(),
                            player_update.position.clone(),
                        );
                        update_event_writer.send(FriendUpdateEvent::new(
                            player_update.player_uuid,
                            player_update.position,
                        ));
                    }
                    Response::PresenceDiff(diff) => {
                        for player in diff.joins {
                            socket.upsert_player(player.clone());
                            if let Some(position) = player.position {
                                update_event_writer
                                    .send(FriendUpdateEvent::new(player.uuid, position));
                            }
                        }
                        for player in diff.leaves {
                            socket.remove_friend(player);
                        }
                    }
                    Response::PresenceState(state) => {
                        socket.upsert_players(state.players.clone());
                        for player in state.players {
                            if let Some(position) = player.position {
                                update_event_writer
                                    .send(FriendUpdateEvent::new(player.uuid, position));
                            }
                        }
                    }
                    Response::Shout(_shout) => (),
                    _ => (),
                }
            }
        },
        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => (),
        Err(_error) => (),
    }
}
