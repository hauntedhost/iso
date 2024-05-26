pub mod client;
pub mod connection;
pub mod message;
pub mod refs;
pub mod request;
pub mod response;
pub mod room;

use self::client::{Client, SocketEvent, SocketStatus};
use self::request::Request;
use self::response::Response;
use crate::player::store::PlayerStore;
use crate::player::systems::FriendUpdateEvent;
use crate::schedule::{StartupSet, UpdateSet};
use crate::socket::connection::{connect_socket, create_channel, get_socket_url};
use bevy::prelude::*;
use bevy::text::BreakLineOn;
use tokio::sync::mpsc::Receiver;

pub const GAME_ROOM: &str = "iso";
pub const HEARTBEAT_INTERVAL_SECS: f32 = 15.0;

#[derive(Debug, Resource)]
pub struct Socket {
    pub handle: ezsockets::Client<Client>,
    pub rx: Receiver<SocketEvent>,
    pub status: Option<SocketStatus>,
    pub last_response: Option<Response>,
    _runtime: tokio::runtime::Runtime,
}

impl Socket {
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

        Self {
            handle,
            rx,
            status: None,
            last_response: None,
            _runtime: runtime,
        }
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
pub struct SocketPlugin {}

impl Default for SocketPlugin {
    fn default() -> Self {
        Self {}
    }
}

impl Plugin for SocketPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Socket::new())
            .insert_resource(HeartbeatTimer::default())
            .register_type::<HeartbeatTimer>()
            .add_systems(Startup, spawn_socket_info.in_set(StartupSet::SpawnEntities))
            .add_systems(
                Update,
                (handle_socket_events, update_socket_info)
                    .chain()
                    .in_set(UpdateSet::AfterEffects),
            )
            .add_systems(Update, send_heartbeat.in_set(UpdateSet::AfterEffects));
    }
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

fn handle_socket_events(
    mut socket: ResMut<Socket>,
    mut store: ResMut<PlayerStore>,
    mut update_event_writer: EventWriter<FriendUpdateEvent>,
) {
    match socket.rx.try_recv() {
        Ok(socket_event) => match socket_event {
            SocketEvent::Close => socket.status = Some(SocketStatus::Closed),
            SocketEvent::Connect => {
                socket.status = Some(SocketStatus::Connected);
                let request = Request::new_join(GAME_ROOM.into(), store.get_player().clone());
                socket.handle.call(request).expect("join error");
            }
            SocketEvent::ConnectFail => socket.status = Some(SocketStatus::ConnectFailed),
            SocketEvent::Disconnect => socket.status = Some(SocketStatus::Disconnected),
            SocketEvent::Response(response) => {
                socket.last_response = Some(response.clone());

                match response {
                    Response::PlayerUpdate(player_update) => {
                        store.update_player_position(
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
                            store.upsert_player(player.clone());
                            if let Some(position) = player.position {
                                update_event_writer
                                    .send(FriendUpdateEvent::new(player.uuid, position));
                            }
                        }
                        for player in diff.leaves {
                            store.remove_friend(player);
                        }
                    }
                    Response::PresenceState(state) => {
                        store.upsert_players(state.players.clone());
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

fn update_socket_info(
    mut socket_info_query: Query<(&mut Text, &SocketInfo), With<SocketInfo>>,
    socket: Res<Socket>,
    store: Res<PlayerStore>,
) {
    let Ok((mut text, socket_info)) = socket_info_query.get_single_mut() else {
        return;
    };

    let socket_status = match &socket.status {
        Some(s) => format!("{:?}", s),
        None => "None".to_string(),
    };

    let player = store.get_player();

    let player_position = match player.position {
        Some(pos) => format!("({:.2}, {:.2})", pos.x, pos.z),
        None => "()".to_string(),
    };

    let friends_info: Vec<String> = store
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

fn send_heartbeat(
    mut heartbeat_timer: ResMut<HeartbeatTimer>,
    time: Res<Time>,
    socket: Res<Socket>,
) {
    heartbeat_timer.timer.tick(time.delta());
    if !heartbeat_timer.timer.just_finished() {
        return;
    }
    let request = Request::new_heartbeat();
    socket.handle.call(request).expect("heartbeat error");
}
