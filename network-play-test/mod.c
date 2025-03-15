#include "modding.h"
#include "global.h"

// MARK: - Imports

RECOMP_IMPORT("mm_network_play", void NP_Init());
RECOMP_IMPORT("mm_network_play", u8 NP_Connect(const char* host, u32 playerId));
RECOMP_IMPORT("mm_network_play", u8 NP_JoinSession(const char* session));
RECOMP_IMPORT("mm_network_play", u8 NP_LeaveCurrentSession());
RECOMP_IMPORT("mm_network_play", void NP_SyncActor(Actor* actor, u32 syncFlags));

RECOMP_IMPORT("*", int recomp_printf(const char* fmt, ...));

RECOMP_IMPORT("ProxyMM_Notifications", void Notifications_Emit(const char* prefix, const char* msg, const char* suffix));

// MARK: - Hooks

RECOMP_HOOK("FileSelect_LoadGame") void OnLoadFile(GameState* thisx) {
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

// MARK: - Events

u8 has_connected = 0;

RECOMP_CALLBACK("*", recomp_on_init)
void init_runtime() {
    NP_Init();
}

RECOMP_CALLBACK("*", recomp_on_play_init)
void on_play_init(PlayState* play) {
    if (has_connected) return;

    has_connected = NP_Connect("wss://echo.websocket.org", 0);

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
