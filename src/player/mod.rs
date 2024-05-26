pub mod player;
pub mod store;
pub mod systems;

use self::{store::PlayerStore, systems::*};
use crate::schedule::{StartupSet, UpdateSet};
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct PlayerPlugin {}

impl Default for PlayerPlugin {
    fn default() -> Self {
        Self {}
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerStore::default())
            .register_type::<PlayerStore>()
            .insert_resource(BroadcastBuffer::default())
            .register_type::<BroadcastBuffer>()
            .add_systems(Startup, spawn_player.in_set(StartupSet::SpawnEntities))
            .add_systems(
                Update,
                (update_player_position, broadcast_player_update)
                    .chain()
                    .in_set(UpdateSet::UserInputEffects),
            )
            .add_systems(
                Update,
                (spawn_friends, despawn_friends, update_friend_positions),
            )
            .add_event::<PlayerUpdateEvent>()
            .add_event::<FriendUpdateEvent>();
    }
}
