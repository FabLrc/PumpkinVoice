use super::secret::Secret;
use uuid::Uuid;

#[derive(Clone)]
pub struct PlayerState {
    pub uuid: Uuid,
    pub name: String,
    pub disconnected: bool,
    pub disabled: bool,
    pub group: Option<Uuid>,
    pub secret: Secret,
    pub socket_addr: Option<std::net::SocketAddr>,
}
