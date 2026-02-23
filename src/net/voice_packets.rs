use bytes::{Buf, BufMut};
use uuid::Uuid;

use crate::state::Secret;
use crate::util::buf_ext::{BufExt, BufMutExt};

#[derive(Clone)]
pub enum VoicePacket {
    Mic(MicPacket),
    PlayerSound(PlayerSoundPacket),
    GroupSound(GroupSoundPacket),
    LocationSound(LocationSoundPacket),
    Authenticate(Box<AuthenticatePacket>),
    AuthenticateAck(AuthenticateAckPacket),
    Ping(PingPacket),
    KeepAlive(KeepAlivePacket),
    ConnectionCheck(ConnectionCheckPacket),
    ConnectionCheckAck(ConnectionCheckAckPacket),
}

impl VoicePacket {
    #[must_use]
    pub fn get_type_id(&self) -> u8 {
        match self {
            Self::Mic(_) => 0x1,
            Self::PlayerSound(_) => 0x2,
            Self::GroupSound(_) => 0x3,
            Self::LocationSound(_) => 0x4,
            Self::Authenticate(_) => 0x5,
            Self::AuthenticateAck(_) => 0x6,
            Self::Ping(_) => 0x7,
            Self::KeepAlive(_) => 0x8,
            Self::ConnectionCheck(_) => 0x9,
            Self::ConnectionCheckAck(_) => 0xA,
        }
    }
}

#[derive(Clone)]
pub struct AuthenticatePacket {
    pub player_uuid: Uuid,
    pub secret: Secret,
}

impl AuthenticatePacket {
    #[must_use]
    pub fn from_bytes(mut buf: impl Buf) -> Self {
        let player_uuid = buf.get_uuid();
        let mut secret_bytes = [0u8; 16];
        secret_bytes.copy_from_slice(&buf.get_uuid().into_bytes()); // Secret mapping
        Self {
            player_uuid,
            secret: Secret::from_bytes(secret_bytes),
        }
    }

    pub fn to_bytes(&self, mut buf: impl BufMut) {
        buf.put_uuid(self.player_uuid);
        buf.put_slice(&self.secret.to_bytes());
    }
}

#[derive(Clone)]
pub struct AuthenticateAckPacket;

impl AuthenticateAckPacket {
    #[must_use]
    pub fn from_bytes(_buf: impl Buf) -> Self {
        Self
    }

    pub fn to_bytes(&self, _buf: impl BufMut) {}
}

#[derive(Clone)]
pub struct PingPacket {
    pub id: Uuid,
    pub timestamp: i64,
}

impl PingPacket {
    #[must_use]
    pub fn from_bytes(mut buf: impl Buf) -> Self {
        Self {
            id: buf.get_uuid(),
            timestamp: buf.get_i64(),
        }
    }

    pub fn to_bytes(&self, mut buf: impl BufMut) {
        buf.put_uuid(self.id);
        buf.put_i64(self.timestamp);
    }
}

#[derive(Clone)]
pub struct MicPacket {
    pub data: Vec<u8>,
    pub sequence_number: i64,
    pub whispering: bool,
}

impl MicPacket {
    #[must_use]
    pub fn from_bytes(mut buf: impl Buf) -> Self {
        let data = buf.get_byte_array();
        let sequence_number = buf.get_i64();
        let whispering = buf.get_u8() != 0;
        Self {
            data,
            sequence_number,
            whispering,
        }
    }

    pub fn to_bytes(&self, mut buf: impl BufMut) {
        buf.put_byte_array(&self.data);
        buf.put_i64(self.sequence_number);
        buf.put_u8(if self.whispering { 1 } else { 0 });
    }
}

#[derive(Clone)]
pub struct PlayerSoundPacket {
    pub channel_id: Uuid,
    pub sender: Uuid,
    pub data: Vec<u8>,
    pub sequence_number: i64,
    pub distance: f32,
    pub whispering: bool,
    pub category: Option<String>,
}

impl PlayerSoundPacket {
    pub fn to_bytes(&self, mut buf: impl BufMut) {
        buf.put_uuid(self.channel_id);
        buf.put_uuid(self.sender);
        buf.put_byte_array(&self.data);
        buf.put_i64(self.sequence_number);
        buf.put_f32(self.distance);

        let mut flags = 0u8;
        if self.whispering {
            flags |= 0b0000_0001;
        }
        if self.category.is_some() {
            flags |= 0b0000_0010;
        }
        buf.put_u8(flags);

        if let Some(cat) = &self.category {
            buf.put_string(cat);
        }
    }
}

#[derive(Clone)]
pub struct GroupSoundPacket {
    pub channel_id: Uuid,
    pub sender: Uuid,
    pub data: Vec<u8>,
    pub sequence_number: i64,
    pub category: Option<String>,
}

impl GroupSoundPacket {
    pub fn to_bytes(&self, mut buf: impl BufMut) {
        buf.put_uuid(self.channel_id);
        buf.put_uuid(self.sender);
        buf.put_byte_array(&self.data);
        buf.put_i64(self.sequence_number);

        let mut flags = 0u8;
        if self.category.is_some() {
            flags |= 0b0000_0010;
        }
        buf.put_u8(flags);

        if let Some(cat) = &self.category {
            buf.put_string(cat);
        }
    }
}

#[derive(Clone)]
pub struct LocationSoundPacket {
    pub channel_id: Uuid,
    pub sender: Uuid,
    pub location: [f64; 3],
    pub data: Vec<u8>,
    pub sequence_number: i64,
    pub distance: f32,
    pub category: Option<String>,
}

impl LocationSoundPacket {
    pub fn to_bytes(&self, mut buf: impl BufMut) {
        buf.put_uuid(self.channel_id);
        buf.put_uuid(self.sender);
        buf.put_f64(self.location[0]);
        buf.put_f64(self.location[1]);
        buf.put_f64(self.location[2]);
        buf.put_byte_array(&self.data);
        buf.put_i64(self.sequence_number);
        buf.put_f32(self.distance);

        let mut flags = 0u8;
        if self.category.is_some() {
            flags |= 0b0000_0010;
        }
        buf.put_u8(flags);

        if let Some(cat) = &self.category {
            buf.put_string(cat);
        }
    }
}

#[derive(Clone)]
pub struct KeepAlivePacket;

impl KeepAlivePacket {
    #[must_use]
    pub fn from_bytes(_buf: impl Buf) -> Self {
        Self
    }

    pub fn to_bytes(&self, _buf: impl BufMut) {}
}

#[derive(Clone)]
pub struct ConnectionCheckPacket;

impl ConnectionCheckPacket {
    #[must_use]
    pub fn from_bytes(_buf: impl Buf) -> Self {
        Self
    }

    pub fn to_bytes(&self, _buf: impl BufMut) {}
}

#[derive(Clone)]
pub struct ConnectionCheckAckPacket;

impl ConnectionCheckAckPacket {
    #[must_use]
    pub fn from_bytes(_buf: impl Buf) -> Self {
        Self
    }

    pub fn to_bytes(&self, _buf: impl BufMut) {}
}
