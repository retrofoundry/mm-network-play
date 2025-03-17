#include "modding.h"
#include "global.h"
#include "recomputils.h"

#define NP_SYNC_POSITION  (1 << 0)
#define NP_SYNC_ROTATION  (1 << 1)
#define NP_SYNC_VELOCITY  (1 << 2)
#define NP_SYNC_SCALE     (1 << 3)
#define NP_SYNC_FLAGS     (1 << 4)
#define NP_SYNC_ALL       (NP_SYNC_POSITION | NP_SYNC_ROTATION | NP_SYNC_VELOCITY | NP_SYNC_SCALE | NP_SYNC_FLAGS)

// MARK: - Imports

RECOMP_IMPORT("mm_network_play", void NP_Init());
RECOMP_IMPORT("mm_network_play", u8 NP_Connect(const char* host));
RECOMP_IMPORT("mm_network_play", u8 NP_JoinSession(const char* session));
RECOMP_IMPORT("mm_network_play", u8 NP_LeaveSession());

RECOMP_IMPORT("mm_network_play", void NP_SyncActor(Actor* actor, u32 syncFlags));
RECOMP_IMPORT("mm_network_play", void NP_ExtendActorSynced(s16 actor_id, u32 size));

RECOMP_IMPORT("ProxyMM_Notifications", void Notifications_Emit(const char* prefix, const char* msg, const char* suffix));

// MARK: - Forward Declarations

void remote_actors_update(PlayState* play);

// MARK: - Events

u8 has_connected = 0;

RECOMP_CALLBACK("*", recomp_on_init)
void init_runtime() {
    NP_Init();
    has_connected = 0;
}

RECOMP_CALLBACK("*", recomp_on_play_init)
void on_play_init(PlayState* play) {
    if (has_connected) return;
    recomp_printf("Connecting to server...\n");
    has_connected = NP_Connect("ws://localhost:8080");

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
        return;
    }

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

// Process remote players on frame
RECOMP_CALLBACK("*", recomp_on_play_main)
void on_play_main(PlayState* play) {
    static u32 last_update = 0;

    if (!has_connected) return;
    remote_actors_update(play);
}

// MARK: - Hooks

RECOMP_HOOK("Player_Init")
void OnPlayerInit(Actor* thisx, PlayState* play) {
    recomp_printf("Player initialized\n");
    NP_SyncActor(thisx, NP_SYNC_POSITION);
}

// MARK: - Remote Player Actor Processing

void remote_actors_update(PlayState* play) {
    recomp_printf("Updating remote player actors...\n");
    remotePlayerCount = NP_GetRemotePlayers(MAX_REMOTE_PLAYERS, remotePlayerData, (char*)remotePlayerIds, 64);
}
