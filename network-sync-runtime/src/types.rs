use n64_recomp::{N64MemoryIO, Vec3f, Vec3s};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, N64MemoryIO)]
pub struct ActorData {
    pub world_position: Vec3f,
    pub shape_rotation: Vec3s,

    // Player Actor specific properties
    pub upper_limb_rot: Vec3s,
    pub joint_table: [Vec3s; 24],
    pub current_mask: i8,
    pub current_shield: i8,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RemoteActorData {
    pub id: String,
    pub data: ActorData,
    pub last_update: std::time::Instant,
}
