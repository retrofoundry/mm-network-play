use n64_recomp::{N64MemoryIO, Vec3f, Vec3s};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, N64MemoryIO)]
pub struct PlayerData {
    pub shapeRotation: Vec3s,
    pub worldPosition: Vec3f,

    // Player Actor specific properties
    pub currentBoots: i8,
    pub currentShield: i8,
    pub _padding: [u8; 2],
    pub jointTable: [Vec3s; 24],
    pub upperLimbRot: Vec3s,
}

#[derive(Debug, Clone)]
pub struct RemotePlayerData {
    pub player_id: String,
    pub data: PlayerData,
    pub last_update: std::time::Instant,
}
