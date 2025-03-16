use n64_recomp::{Vec3f, Vec3s};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pub currentBoots: i8,
    pub currentShield: i8,
    pub jointTable: [Vec3s; 24], // Might need to increase this in the future
    pub upperLimbRot: Vec3s,
    pub shapeRotation: Vec3s,
    pub worldPosition: Vec3f,
}

#[derive(Debug, Clone)]
pub struct RemotePlayerData {
    pub player_id: String,
    pub data: PlayerData,
    pub last_update: std::time::Instant,
}
