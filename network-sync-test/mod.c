#include "modding.h"
#include "global.h"
#include "recomputils.h"
#include "string.h"

#ifdef _DEBUG
    #define SERVER_URL "ws://localhost:8080"
#else
    #define SERVER_URL "wss://mm-net.dcvz.io"
#endif

// MARK: - Imports

RECOMP_IMPORT("mm_network_sync", void NS_Init());
RECOMP_IMPORT("mm_network_sync", u8 NS_Connect(const char* host));
RECOMP_IMPORT("mm_network_sync", u8 NS_JoinSession(const char* session));
RECOMP_IMPORT("mm_network_sync", u8 NS_LeaveSession());
RECOMP_IMPORT("mm_network_sync", const char* NS_GetActorNetworkId(Actor *actor));
RECOMP_IMPORT("mm_network_sync", u32 NS_GetRemotePlayerIDs(u32 maxPlayers, char* idsBuffer, u32 idBufferSize));
RECOMP_IMPORT("mm_network_sync", u32 NS_GetRemotePlayerData(const char* playerID, void* dataBuffer));

RECOMP_IMPORT("mm_network_sync", void NS_SyncActor(Actor* actor, const char* playerID));
RECOMP_IMPORT("mm_network_sync", void NS_ExtendActorSynced(s16 actor_id, u32 size));

RECOMP_IMPORT("ProxyMM_Notifications", void Notifications_Emit(const char* prefix, const char* msg, const char* suffix));
RECOMP_IMPORT("ProxyMM_CustomActor", s16 CustomActor_Register(ActorProfile* profile));

// MARK: - Forward Declarations

void remote_actors_update(PlayState* play);

// MARK: - Custom Actors

extern ActorProfile RemotePlayer_InitVars;
s16 ACTOR_REMOTE_PLAYER = ACTOR_ID_MAX;

// MARK: - Events

u8 has_connected = 0;

RECOMP_CALLBACK("*", recomp_on_init)
void init_runtime() {
    has_connected = 0;

    NS_Init();
    ACTOR_REMOTE_PLAYER = CustomActor_Register(&RemotePlayer_InitVars);
}

RECOMP_CALLBACK("*", recomp_on_play_init)
void on_play_init(PlayState* play) {
    if (has_connected) return;
    recomp_printf("Connecting to server...\n");
    has_connected = NS_Connect(SERVER_URL);

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

    u8 result = NS_JoinSession("test");
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
    NS_SyncActor(thisx, NULL);
}

// MARK: - Remote Player Actor Processing

#define MAX_REMOTE_PLAYERS 32 // matches the mod's MAX_SYNCED_ACTORS
static char remotePlayerIds[MAX_REMOTE_PLAYERS][64];
static u32 remotePlayerCount = 0;

// Checks whether we need to create or destroy actors
void remote_actors_update(PlayState* play) {
    remotePlayerCount = NS_GetRemotePlayerIDs(MAX_REMOTE_PLAYERS, (char*)remotePlayerIds, 64);
    recomp_printf("Remote player count: %d\n", remotePlayerCount);

    // Create actors for new remote players (only if we have any)
    for (u32 i = 0; i < remotePlayerCount; i++) {
        // 1. Check if player already has an actor
        bool remoteActorAlreadyCreated = false;
        Actor* actor = play->actorCtx.actorLists[ACTORCAT_PLAYER].first;

        // Find actor with given ID
        while (actor != NULL) {
            if (actor->id == ACTOR_REMOTE_PLAYER) {
                const char* actorNetworkId = NS_GetActorNetworkId(actor);
                const char* playerId = remotePlayerIds[i];

                if (actorNetworkId != NULL && strcmp(actorNetworkId, playerId) == 0) {
                    remoteActorAlreadyCreated = true;
                    break;
                }
            }

            actor = actor->next;
        }

        // 2. If actor not found, create new actor
        if (!remoteActorAlreadyCreated) {
            const char* playerId = remotePlayerIds[i];
            recomp_printf("Creating actor for player %s\n", playerId);
            actor = Actor_SpawnAsChildAndCutscene(&play->actorCtx, play, ACTOR_REMOTE_PLAYER, -9999.0f, -9999.0f, -9999.0f, 0, 0, 0, 0, 0, 0, 0);
            NS_SyncActor(actor, playerId);
        }
    }

    // Check for players that no longer exist and remove their actors
    Actor* actor = play->actorCtx.actorLists[ACTORCAT_PLAYER].first;
    while (actor != NULL) {
        Actor* next = actor->next; // Save next pointer as we may delete this actor

        if (actor->id == ACTOR_REMOTE_PLAYER) {
            const char* actorNetworkId = NS_GetActorNetworkId(actor);
            if (actorNetworkId == NULL) {
                Actor_Kill(actor);
                recomp_printf("Removed remote player with NULL ID\n");
            } else {
                bool stillExists = false;

                // Only check if there are remote players to compare against
                if (remotePlayerCount > 0) {
                    for (u32 i = 0; i < remotePlayerCount; i++) {
                        if (strcmp(actorNetworkId, remotePlayerIds[i]) == 0) {
                            stillExists = true;
                            break;
                        }
                    }
                } else {
                    // If there are no remote players, this actor definitely doesn't exist anymore
                    stillExists = false;
                    recomp_printf("No remote players exist, removing actor with ID %s\n", actorNetworkId);
                }

                if (!stillExists) {
                    Actor_Kill(actor);
                    recomp_printf("Removed remote player %s\n", actorNetworkId);
                }
            }
        }

        actor = next;
    }
}
