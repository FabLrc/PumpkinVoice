use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::state::group::Group;
use crate::state::player::PlayerState;
use crate::state::secret::Secret;

pub struct StateManager {
    states: RwLock<HashMap<Uuid, PlayerState>>,
    groups: RwLock<HashMap<Uuid, Group>>,
    categories: RwLock<HashMap<String, crate::net::VolumeCategory>>,
}

impl StateManager {
    #[must_use]
    pub fn new() -> Self {
        let mut cats = HashMap::new();
        // Insert a demo category so the mod populates the UI
        cats.insert(
            "radio".to_string(),
            crate::net::VolumeCategory {
                id: "radio".to_string(),
                name: "Radio Team".to_string(),
                description: Some("Global broadcast".to_string()),
            },
        );

        Self {
            states: RwLock::new(HashMap::new()),
            groups: RwLock::new(HashMap::new()),
            categories: RwLock::new(cats),
        }
    }

    pub async fn add_player(&self, uuid: Uuid, name: String) -> Secret {
        let secret = Secret::generate();
        let state = PlayerState {
            uuid,
            name,
            disconnected: false,
            disabled: false,
            group: None,
            secret: secret.clone(),
            socket_addr: None,
        };
        self.states.write().await.insert(uuid, state);
        secret
    }

    pub async fn remove_player(&self, uuid: &Uuid) {
        self.states.write().await.remove(uuid);
    }

    pub async fn get_player(&self, uuid: &Uuid) -> Option<PlayerState> {
        self.states.read().await.get(uuid).cloned()
    }

    pub async fn update_state(&self, uuid: &Uuid, disconnected: bool, disabled: bool) {
        if let Some(state) = self.states.write().await.get_mut(uuid) {
            state.disconnected = disconnected;
            state.disabled = disabled;
        }
    }

    pub async fn update_player_addr(&self, uuid: &Uuid, addr: std::net::SocketAddr) {
        if let Some(state) = self.states.write().await.get_mut(uuid) {
            state.socket_addr = Some(addr);
        }
    }

    pub async fn get_all_players(&self) -> Vec<PlayerState> {
        self.states.read().await.values().cloned().collect()
    }

    pub async fn get_keep_alive_targets(&self) -> Vec<(std::net::SocketAddr, Secret)> {
        self.states
            .read()
            .await
            .values()
            .filter_map(|p| p.socket_addr.map(|addr| (addr, p.secret.clone())))
            .collect()
    }

    pub async fn add_group(&self, group: Group) {
        self.groups.write().await.insert(group.id, group);
    }

    pub async fn get_group(&self, id: &Uuid) -> Option<Group> {
        self.groups.read().await.get(id).cloned()
    }

    pub async fn get_group_by_name(&self, name: &str) -> Option<Group> {
        self.groups
            .read()
            .await
            .values()
            .find(|g| g.name == name)
            .cloned()
    }

    pub async fn get_all_groups(&self) -> Vec<Group> {
        self.groups.read().await.values().cloned().collect()
    }

    pub async fn get_categories(&self) -> Vec<crate::net::VolumeCategory> {
        let guard = self.categories.read().await;
        // Clone each category manually
        guard
            .values()
            .map(|c| crate::net::VolumeCategory {
                id: c.id.clone(),
                name: c.name.clone(),
                description: c.description.clone(),
            })
            .collect()
    }

    pub async fn remove_group(&self, id: &Uuid) {
        self.groups.write().await.remove(id);
    }

    pub async fn set_player_group(&self, player_uuid: &Uuid, group_id: Option<Uuid>) {
        if let Some(state) = self.states.write().await.get_mut(player_uuid) {
            state.group = group_id;
        }
    }

    pub async fn remove_if_empty(&self, group_id: &Uuid) -> bool {
        let players = self.states.read().await;
        // Check if any player is in this group
        let has_players = players.values().any(|p| p.group == Some(*group_id));
        if !has_players {
            let mut groups = self.groups.write().await;
            if let Some(g) = groups.get(group_id) {
                if !g.persistent {
                    groups.remove(group_id);
                    return true;
                }
            }
        }
        false
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}
