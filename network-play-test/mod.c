#include "modding.h"
#include "global.h"
#include "recomputils.h"
#include "z_remote_player.h"

// MARK: - Imports

RECOMP_IMPORT("mm_network_play", void NP_Init());
RECOMP_IMPORT("mm_network_play", u8 NP_Connect(const char* host));
RECOMP_IMPORT("mm_network_play", u8 NP_JoinSession(const char* session));
RECOMP_IMPORT("mm_network_play", u8 NP_LeaveSession());
RECOMP_IMPORT("mm_network_play", u32 NP_GetRemotePlayerIDs(u32 maxPlayers, char* idsBuffer, u32 idBufferSize));
RECOMP_IMPORT("mm_network_play", u32 NP_GetRemotePlayerData(const char* playerID, void* dataBuffer));

RECOMP_IMPORT("mm_network_play", void NP_SyncActor(Actor* actor));
RECOMP_IMPORT("mm_network_play", void NP_ExtendActorSynced(s16 actor_id, u32 size));

RECOMP_IMPORT("ProxyMM_Notifications", void Notifications_Emit(const char* prefix, const char* msg, const char* suffix));
RECOMP_IMPORT("ProxyMM_CustomActor", s16 CustomActor_Register(ActorProfile* profile));

// MARK: - Forward Declarations

void remote_actors_update(PlayState* play);

// MARK: Structs

// Direct Copy from mm_network_play (can we export this somehow?)
typedef struct {
    s8 currentBoots;
    s8 currentShield;
    u8 _padding[2]; // Add padding for alignment
    Vec3s jointTable[24]; // Might need to increase this in the future
    Vec3s upperLimbRot;
    Vec3s shapeRotation;
    Vec3f worldPosition;
} PlayerSyncData;

// MARK: - Custom Actors

extern ActorProfile RemotePlayer_InitVars;
s16 ACTOR_REMOTE_PLAYER = ACTOR_ID_MAX;

// MARK: - Events

u8 has_connected = 0;

RECOMP_CALLBACK("*", recomp_on_init)
void init_runtime() {
    has_connected = 0;

    NP_Init();
    ACTOR_REMOTE_PLAYER = CustomActor_Register(&RemotePlayer_InitVars);
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
    NP_SyncActor(thisx);
}

// MARK: - Remote Player Actor Processing

#define MAX_REMOTE_PLAYERS 32 // matches the mod's MAX_SYNCED_ACTORS
static char remotePlayerIds[MAX_REMOTE_PLAYERS][64];
static u32 remotePlayerCount = 0;

void remote_actors_update(PlayState* play) {
    recomp_printf("Updating remote player actors...\n");

    remotePlayerCount = NP_GetRemotePlayerIDs(MAX_REMOTE_PLAYERS, (char*)remotePlayerIds, 64);
    recomp_printf("Remote player count: %d\n", remotePlayerCount);
    if (remotePlayerCount == 0) {
        return;
    }

    for (u32 i = 0; i < remotePlayerCount; i++) {
        // 1. Check if player already has an actor
        bool remoteActorAlreadyCreated = false;
        Actor* actor = play->actorCtx.actorLists[ACTORCAT_NPC].first; // id needs to be updated

        // Find actor with given ID
        while (actor != NULL) {
            if (actor->id == ACTOR_REMOTE_PLAYER) {
                RemotePlayer* remote = (RemotePlayer*)actor;

                // Check if this is the same player by ID via extension data?
                // If we find it then break so we can complete this part
            }
            actor = actor->next;
        }

        // 2. If actor not found, create new actor
        if (!remoteActorAlreadyCreated) {
            PlayerSyncData playerData;
            const char* playerId = remotePlayerIds[i];
            if (NP_GetRemotePlayerData(playerId, &playerData)) {
                // 1. Spawn Actor
                // 2. Call NP_SyncActor with actor and the uuid
                // 3. Let the sync engine handle the rest

                // This is just for debugging, we don't care about positions here.
                recomp_printf("Remote player %d ID: %s is at position (%f, %f, %f)\n", i, playerId, playerData.worldPosition.x, playerData.worldPosition.y, playerData.worldPosition.z);
            }
        }
    }

    // Check for players that no longer exist and remove their actors
    Actor* actor = play->actorCtx.actorLists[ACTORCAT_NPC].first; // id needs to be updated
    while (actor != NULL) {
        Actor* next = actor->next; // Save next pointer as we may delete this actor
        if (actor->id == ACTOR_REMOTE_PLAYER) {
            RemotePlayer* remote = (RemotePlayer*)actor;
            // 1. Grab ID via extension data
            // 2. Check if ID exists in remotePlayerIds array
            // 3. If not, remove actor

            bool stillExists = false;

            for (u32 i = 0; i < remotePlayerCount; i++) {
                // if (strcmp(remote->player_id, remotePlayerIds[i]) == 0) {
                //     stillExists = true;
                //     break;
                // }
            }

            if (!stillExists) {
                Actor_Kill(actor);
                // recomp_printf("Removed remote player %s\n", remote->player_id);
            }
        }

        actor = next;
    }
}
