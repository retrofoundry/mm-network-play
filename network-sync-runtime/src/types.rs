use n64_recomp::{N64MemoryIO, Vec3f, Vec3s};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, N64MemoryIO)]
pub struct ActorData {
    pub worldPosition: Vec3f,
    pub shapeRotation: Vec3s,

    // Player Actor specific properties
    pub upperLimbRot: Vec3s,
    pub jointTable: [Vec3s; 24],
    pub currentBoots: i8,
    pub currentShield: i8,
}

#[derive(Debug, Clone)]
pub struct RemoteActorData {
    pub id: String,
    pub data: ActorData,
    pub last_update: std::time::Instant,
}
