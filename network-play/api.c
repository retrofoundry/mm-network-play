#include "modding.h"
#include "global.h"
#include "recomputils.h"
#include "recompconfig.h"
#include "recompui.h"
#include "z64recomp_api.h"

// MARK: - Imports

RECOMP_IMPORT(".", void NetworkPlayInit());
RECOMP_IMPORT(".", u8 NetworkPlayConnect(const char* host));
RECOMP_IMPORT(".", u8 NetworkPlayJoinSession(const char* session));
RECOMP_IMPORT(".", u8 NetworkPlayLeaveSession());

// MARK: - Events

RECOMP_CALLBACK("*", recomp_after_actor_update)
void on_actor_update(PlayState* play, Actor* actor) {
    // if (actor->id == 0) {
    //     recomp_printf("OnActorUpdate: player at %f %f %f\n", actor->world.pos.x, actor->world.pos.y, actor->world.pos.z);
    // } else {
    //     recomp_printf("OnActorUpdate: actor %u at %f %f %f\n", actor->id, actor->world.pos.x, actor->world.pos.y, actor->world.pos.z);
    // }
}

// MARK: - API

RECOMP_EXPORT void NP_Init() {
    NetworkPlayInit();
}

RECOMP_EXPORT u8 NP_Connect(const char* host, u32 playerId) {
    return NetworkPlayConnect(host);
}

RECOMP_EXPORT u8 NP_JoinSession(const char* session) {
    return NetworkPlayJoinSession(session);
}

RECOMP_EXPORT u8 NP_LeaveSession() {
    return NetworkPlayLeaveSession();
}

// MARK: - Syncing

RECOMP_EXPORT void NP_SyncActor(Actor* actor, u32 id, u32 syncFlags) {
    recomp_printf("Syncing actor %u of type %p with flags %u\n", id, actor, syncFlags);
}

// MARK: Actor Extensions

RECOMP_EXPORT ActorExtensionId NP_ExtendActorSynced(s16 actor_id, u32 size) {
    return z64recomp_extend_actor(actor_id, size);
}

// RECOMP_EXPORT void NP_WriteActorSyncedData(Actor* actor, ActorExtensionId extension, const void* data) {
//     void* actor_data = z64recomp_get_extended_actor_data(actor, extension);
//     if (actor_data) {
//         memcpy(actor_data, data, z64recomp_get_actor_data_size(extension));
//         // push to network
//     }
// }

RECOMP_EXPORT void* NP_GetExtendedActorSyncedData(Actor* actor, ActorExtensionId extension) {
    return z64recomp_get_extended_actor_data(actor, extension);
}

// Sync flags for different actor properties
#define NP_SYNC_POSITION  (1 << 0)
#define NP_SYNC_ROTATION  (1 << 1)
#define NP_SYNC_VELOCITY  (1 << 2)
#define NP_SYNC_SCALE     (1 << 3)
#define NP_SYNC_FLAGS     (1 << 4)
#define NP_SYNC_ALL       (NP_SYNC_POSITION | NP_SYNC_ROTATION | NP_SYNC_VELOCITY | NP_SYNC_SCALE | NP_SYNC_FLAGS)
