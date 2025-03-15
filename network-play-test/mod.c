#include "modding.h"
#include "global.h"
#include "recomputils.h"

// MARK: - Imports

RECOMP_IMPORT("mm_network_play", void NP_Init());
RECOMP_IMPORT("mm_network_play", u8 NP_Connect(const char* host, u32 playerId));
RECOMP_IMPORT("mm_network_play", u8 NP_JoinSession(const char* session));
RECOMP_IMPORT("mm_network_play", u8 NP_LeaveCurrentSession());

RECOMP_IMPORT("mm_network_play", void NP_SyncActor(Actor* actor, u32 id, u32 syncFlags));
RECOMP_IMPORT("mm_network_play", void NP_ExtendActorSynced(s16 actor_id, u32 size));

RECOMP_IMPORT("ProxyMM_Notifications", void Notifications_Emit(const char* prefix, const char* msg, const char* suffix));

// MARK: - Events

u8 has_connected = 0;

RECOMP_CALLBACK("*", recomp_on_init)
void init_runtime() {
    NP_Init();

    // Add entry to store rupees
    // NP_ExtendActorSynced(0, sizeof(s16));
    // gSaveContext.save.saveInfo.playerData.rupees
}

RECOMP_CALLBACK("*", recomp_on_play_init)
void on_play_init(PlayState* play) {
    if (has_connected) return;

    has_connected = NP_Connect("ws://localhost:8080", 0);

    if (has_connected) {
        Notifications_Emit(
            "", // Prefix (Purple)
            "Connected to server", // Main Message (white)
            "" // Suffix (Blue)
        );
    } else {
        Notifications_Emit(
            "Failed to connect to server", // Prefix (Purple)
            "", // Main Message (white)
            "" // Suffix (Blue)
        );
    }
}

// MARK: - Hooks

RECOMP_HOOK("FileSelect_LoadGame")
void OnLoadFile(GameState* thisx) {
    u8 result = NP_JoinSession("test");
    if (result) {
        Notifications_Emit(
            "", // Prefix (Purple)
            "Joined session", // Main Message (white)
            "" // Suffix (Blue)
        );
    } else {
        Notifications_Emit(
            "Failed to join session", // Prefix (Purple)
            "", // Main Message (white)
            "" // Suffix (Blue)
        );
    }
}
