use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use uuid::Uuid;

use common::{GameSettings, user::UserId};

use crate::network::ConnectionInfo;

const PENDING_PLAYER_TIMEOUT: Duration = Duration::from_secs(10);
const SESSION_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(60 * 10);

#[derive(Debug, Resource)]
pub struct GameServerInfo {
    pub server_id: Uuid,
    pub connection_info: ConnectionInfo,
}

impl GameServerInfo {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            server_id: Uuid::new_v4(),
            connection_info: ConnectionInfo::default(),
        }
    }
}

#[derive(Debug, Resource)]
pub struct GameSessionInfo {
    pub session_id: Uuid,
    pub max_players: u16,

    pending_player_count: usize,
    active_player_count: usize,

    clients: HashMap<ClientId, UserId>,

    shutdown_timer: Timer,
}

impl GameSessionInfo {
    pub fn new(
        commands: &mut Commands,
        session_id: Uuid,
        settings: &GameSettings,
        pending_player_ids: impl AsRef<[UserId]>,
    ) -> Self {
        let mut this = Self {
            session_id,
            max_players: settings.max_players,
            pending_player_count: 0,
            active_player_count: 0,
            clients: HashMap::with_capacity(settings.max_players as usize),
            shutdown_timer: Timer::new(SESSION_SHUTDOWN_TIMEOUT, TimerMode::Once),
        };
        this.shutdown_timer.pause();

        for pending_player_id in pending_player_ids.as_ref().iter() {
            this.reserve_player(commands, *pending_player_id);
        }

        this
    }

    #[inline]
    pub fn player_count(&self) -> usize {
        self.pending_player_count + self.active_player_count
    }

    #[inline]
    pub fn has_client(&self, client_id: &ClientId) -> bool {
        self.clients.contains_key(client_id)
    }

    #[inline]
    pub fn update_shutdown_timer(&mut self, delta: Duration) -> bool {
        self.shutdown_timer.tick(delta);
        self.shutdown_timer.finished()
    }

    pub fn reserve_player(&mut self, commands: &mut Commands, pending_player_id: UserId) {
        if self.player_count() + 1 > self.max_players as usize {
            warn!(
                "not reserving player slot for {}, max players {} exceeded!",
                pending_player_id, self.max_players
            );
            return;
        }

        info!("reserving player slot {}", pending_player_id);

        commands.spawn(PendingPlayer::new(pending_player_id));
        self.pending_player_count += 1;

        self.shutdown_timer.pause();
    }

    pub fn pending_player_timeout(
        &mut self,
        commands: &mut Commands,
        pending_player: Entity,
        pending_player_id: UserId,
    ) {
        info!("pending player {} timeout", pending_player_id);

        commands.entity(pending_player).despawn_recursive();
        self.pending_player_count -= 1;

        if self.player_count() == 0 {
            self.shutdown_timer.reset();
            self.shutdown_timer.unpause();
        }
    }

    pub fn client_connected<'a>(
        &mut self,
        commands: &mut Commands,
        user_id: UserId,
        client_id: ClientId,
        mut pending_players: impl Iterator<Item = (Entity, &'a PendingPlayer)>,
    ) -> bool {
        if let Some(pending_player) = pending_players.find_map(|v| {
            if v.1.user_id == user_id {
                Some(v.0)
            } else {
                None
            }
        }) {
            info!("activating player slot {} for {:?}", user_id, client_id);

            commands.entity(pending_player).despawn_recursive();
            self.pending_player_count -= 1;

            commands.spawn(ActivePlayer::new(user_id));
            self.active_player_count += 1;

            self.clients.insert(client_id, user_id);

            self.shutdown_timer.pause();

            true
        } else {
            false
        }
    }

    pub fn client_disconnected<'a>(
        &mut self,
        commands: &mut Commands,
        client_id: &ClientId,
        mut pending_players: impl Iterator<Item = (Entity, &'a PendingPlayer)>,
        mut active_players: impl Iterator<Item = (Entity, &'a ActivePlayer)>,
    ) {
        if let Some(user_id) = self.clients.remove(client_id) {
            if let Some(pending_player) = pending_players.find_map(|v| {
                if v.1.user_id == user_id {
                    Some(v.0)
                } else {
                    None
                }
            }) {
                info!("pending player {} disconnected ?", user_id);

                commands.entity(pending_player).despawn_recursive();
                self.pending_player_count -= 1;
            }

            let active_player = active_players.find_map(|v| {
                if v.1.user_id == user_id {
                    Some(v.0)
                } else {
                    None
                }
            });
            if let Some(active_player) = active_player {
                info!("active player {} disconnected ?", user_id);

                commands.entity(active_player).despawn_recursive();
                self.active_player_count -= 1;
            }
        }

        if self.player_count() == 0 {
            self.shutdown_timer.reset();
            self.shutdown_timer.unpause();
        }
    }
}

#[derive(Debug, Component)]
pub struct PendingPlayer {
    pub user_id: UserId,
    timer: Timer,
}

impl PendingPlayer {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            timer: Timer::new(PENDING_PLAYER_TIMEOUT, TimerMode::Once),
        }
    }

    pub fn is_timeout(&mut self, delta: Duration) -> bool {
        self.timer.tick(delta);
        self.timer.finished()
    }
}

#[derive(Debug, Component)]
pub struct ActivePlayer {
    pub user_id: UserId,
}

impl ActivePlayer {
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }
}
