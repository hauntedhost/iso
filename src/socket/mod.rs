pub mod client;
pub mod connection;
pub mod message;
pub mod names;
pub mod refs;
pub mod request;
pub mod response;
pub mod room;
pub mod user;

use self::client::{Client, SocketEvent, SocketStatus};
use self::request::Request;
use self::response::Response;
use self::user::User;
use crate::socket::connection::{connect_socket, create_channel, get_socket_url};
use bevy::prelude::*;
use bevy::text::BreakLineOn;
use bevy::utils::HashMap;
use tokio::sync::mpsc::Receiver;

pub const GAME_ROOM: &str = "iso";
pub const HEARTBEAT_INTERVAL: f32 = 15.0;

#[derive(Component)]
struct SocketInfo {
    text_section: TextSection,
}

#[derive(Resource, Debug)]
pub struct HeartbeatTimer {
    timer: Timer,
}

impl Default for HeartbeatTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(HEARTBEAT_INTERVAL, TimerMode::Repeating),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config;

impl Default for Config {
    fn default() -> Self {
        Self {}
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
        app.insert_resource(User::new_from_env_or_generate())
            .insert_resource(GameSocket::new())
            .insert_resource(HeartbeatTimer::default())
            .add_systems(Startup, spawn_socket_info)
            .add_systems(Update, hello_socket)
            .add_systems(Update, update_socket_info)
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
        .spawn(NodeBundle {
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
        })
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

    let users: Vec<String> = socket
        .users
        .iter()
        .map(|(_uuid, user)| {
            let position = match user.position {
                Some(pos) => format!("({:.2}, {:.2})", pos.x, pos.z),
                None => "()".to_string(),
            };
            format!("@{} {}", user.username, position)
        })
        .collect();

    let status = match &socket.status {
        Some(s) => format!("{:?}", s),
        None => "None".to_string(),
    };

    let info_text = format!("status={status} users={:?}", users);

    let mut text_section = socket_info.text_section.clone();
    text_section.value = info_text;
    text.sections = vec![text_section];
}

#[derive(Debug, Resource)]
pub struct GameSocket {
    pub users: HashMap<String, User>,
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

        Self {
            handle,
            rx,
            status: None,
            last_response: None,
            users: HashMap::new(),
            _runtime: runtime,
        }
    }

    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.uuid.clone(), user.clone());
    }

    pub fn remove_user(&mut self, user: User) {
        self.users.remove(&user.uuid);
    }

    pub fn set_users(&mut self, users: Vec<User>) {
        for user in users {
            self.add_user(user);
        }
    }

    pub fn update_user_position(&mut self, user: User, position: Vec3) {
        if let Some(user) = self.users.get_mut(&user.uuid) {
            user.position = Some(position);
        }
    }
}

fn hello_socket(mut socket: ResMut<GameSocket>, user: Res<User>) {
    match socket.rx.try_recv() {
        Ok(socket_event) => match socket_event {
            SocketEvent::Close => socket.status = Some(SocketStatus::Closed),
            SocketEvent::Connect => {
                socket.status = Some(SocketStatus::Connected);
                let request = Request::new_join(GAME_ROOM.into(), user.clone());
                socket.handle.call(request).expect("join error");
            }
            SocketEvent::ConnectFail => socket.status = Some(SocketStatus::ConnectFailed),
            SocketEvent::Disconnect => socket.status = Some(SocketStatus::Disconnected),
            SocketEvent::Response(response) => {
                info!("response={:?}", &response);
                socket.last_response = Some(response.clone());

                match response {
                    Response::PresenceDiff(diff) => {
                        for user in diff.joins {
                            socket.add_user(user);
                        }
                        for user in diff.leaves {
                            socket.remove_user(user);
                        }
                    }
                    Response::PresenceState(state) => {
                        socket.set_users(state.users);
                    }
                    Response::Shout(shout) => {
                        if let Some(position) = shout.position {
                            socket.update_user_position(shout.user, position);
                        }
                    }
                    _ => (),
                }
            }
        },
        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => (),
        Err(_error) => (),
    }
}
