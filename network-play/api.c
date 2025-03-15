#include "modding.h"
#include "global.h"

// MARK: - Imports

RECOMP_IMPORT(".", void NetworkPlayInit());
RECOMP_IMPORT(".", void NetworkPlaySetPlayerId(u32 id));
RECOMP_IMPORT(".", u8 NetworkPlayConnect(const char* host));
// RECOMP_IMPORT(".", u8 NetworkPlayJoinSession(const char* session));
RECOMP_IMPORT(".", u8 NetworkPlayLeaveCurrentSession());

RECOMP_IMPORT("*", int recomp_printf(const char* fmt, ...));

// MARK: - Events

RECOMP_CALLBACK("*", recomp_after_actor_update) void on_actor_update(PlayState* play, Actor* actor) {
    if (actor->id == 0) {
        recomp_printf("OnActorUpdate: player at %f %f %f\n", actor->world.pos.x, actor->world.pos.y, actor->world.pos.z);
    } else {
        recomp_printf("OnActorUpdate: actor %u at %f %f %f\n", actor->id, actor->world.pos.x, actor->world.pos.y, actor->world.pos.z);
    }
}

// MARK: - API

RECOMP_EXPORT void NP_Init() {
    NetworkPlayInit();
}

RECOMP_EXPORT u8 NP_Connect(const char* host, u32 playerId) {
    NetworkPlaySetPlayerId(playerId);
    return NetworkPlayConnect(host);
}

RECOMP_EXPORT u8 NP_JoinSession(const char* session) {
    // return NetworkPlayJoinSession(session);
    return 0;
}

RECOMP_EXPORT u8 NP_LeaveCurrentSession() {
    return NetworkPlayLeaveCurrentSession();
}

RECOMP_EXPORT void NP_SyncActor(Actor* actor, u32 syncFlags) {
    recomp_printf("Syncing actor %p with flags %u\n", actor, syncFlags);
}


// Sync flags for different actor properties
#define NP_SYNC_POSITION  (1 << 0)
#define NP_SYNC_ROTATION  (1 << 1)
#define NP_SYNC_VELOCITY  (1 << 2)
#define NP_SYNC_SCALE     (1 << 3)
#define NP_SYNC_FLAGS     (1 << 4)
#define NP_SYNC_ALL       (NP_SYNC_POSITION | NP_SYNC_ROTATION | NP_SYNC_VELOCITY | NP_SYNC_SCALE | NP_SYNC_FLAGS)
