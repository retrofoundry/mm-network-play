#include "modding.h"
#include "global.h"
#include "recomputils.h"
#include "recompconfig.h"
#include "recompui.h"
#include "z64recomp_api.h"
#include <stdint.h>
#include <string.h>

// MARK: - Actor Extension

// Extension ID for network player data
const ActorExtensionId ACTOR_EXTENSION_INVALID = UINT32_MAX;
static ActorExtensionId gNetworkPlayerExtension = ACTOR_EXTENSION_INVALID;

#define MAX_ACTOR_CATEGORIES 12
const u32 MAX_SYNCED_ACTORS = 32;
static u8 gSyncedActorCategories[MAX_ACTOR_CATEGORIES] = {0};  // Bitset for categories with synced actors

// Structure to hold network-specific data for each actor
typedef struct {
    char player_id[64];    // UUID string for this actor
    u8 is_synced;          // Flag indicating if actor is being synced
} NetworkPlayerData;

static NetworkPlayerData* GetActorNetworkData(Actor* actor) {
    if (gNetworkPlayerExtension == ACTOR_EXTENSION_INVALID) {
        return NULL;
    }

    return (NetworkPlayerData*)z64recomp_get_extended_actor_data(actor, gNetworkPlayerExtension);
}

// MARK: - Struct

typedef struct {
    s8 currentBoots;
    s8 currentShield;
    u8 _padding[2]; // Add padding for alignment
    Vec3s jointTable[24]; // Might need to increase this in the future
    Vec3s upperLimbRot;
    Vec3s shapeRotation;
    Vec3f worldPosition;
} PlayerSyncData;

// MARK: - Imports

RECOMP_IMPORT(".", void NetworkPlayInit());
RECOMP_IMPORT(".", u8 NetworkPlayConnect(const char* host));
RECOMP_IMPORT(".", u8 NetworkPlayJoinSession(const char* session));
RECOMP_IMPORT(".", u8 NetworkPlayLeaveSession());
RECOMP_IMPORT(".", u8 NetworkPlayGetPlayerId(char* buffer, u32 bufferSize));
RECOMP_IMPORT(".", void NetworkPlaySendPlayerSync(PlayerSyncData* data));
RECOMP_IMPORT(".", u32 NetworkPlayGetRemotePlayerIDs(u32 maxPlayers, char* idsBuffer, u32 idBufferSize));
RECOMP_IMPORT(".", u32 NetworkPlayGetRemotePlayerData(const char* player_id, PlayerSyncData* data));

// MARK: - Events

RECOMP_CALLBACK("*", recomp_after_actor_update)
void on_actor_update(PlayState* play, Actor* actor) {
    NetworkPlayerData* netData = GetActorNetworkData(actor);

    // Skip actors that aren't being synced or aren't the main player
    if (netData == NULL || !netData->is_synced || actor->id != 0) {
        return;
    }

    // Sync the player movement data to the server
    Player* player = (Player*)actor;
    PlayerSyncData* syncData = recomp_alloc(sizeof(PlayerSyncData) + sizeof(Vec3s) * 23); // For 24 joints

    syncData->currentBoots = player->currentBoots;
    syncData->currentShield = player->currentShield;

    // Copy each joint individually (assuming jointTable is an array of 24 Vec3s)
    for (int i = 0; i < 24; i++) {
        Math_Vec3s_Copy(&syncData->jointTable[i], &player->skelAnime.jointTable[i]);
    }

    Math_Vec3s_Copy(&syncData->upperLimbRot, &player->upperLimbRot);
    Math_Vec3s_Copy(&syncData->shapeRotation, &actor->shape.rot);
    Math_Vec3f_Copy(&syncData->worldPosition, &actor->world.pos);

    NetworkPlaySendPlayerSync(syncData);

    recomp_free(syncData);
}

RECOMP_CALLBACK("*", recomp_on_play_main)
void on_play_main(PlayState* play) {
    //
    // Actor* actor = play->actorCtx.actorLists[ACTORCAT_NPC].first;

    PlayerSyncData remote_data;

    char ids_buffer[MAX_SYNCED_ACTORS * 64];
    u32 player_count = NetworkPlayGetRemotePlayerIDs(MAX_SYNCED_ACTORS, ids_buffer, 64);

    for (u32 i = 0; i < MAX_ACTOR_CATEGORIES; i++) {
        // Skip categories that don't have any synced actors
        if (gSyncedActorCategories[i] == 0) {
            continue;
        }

        Actor* actor = play->actorCtx.actorLists[i].first;

        while (actor != NULL) {
            NetworkPlayerData* net_data = GetActorNetworkData(actor);
            Actor* next_actor = actor->next; // Save next pointer before any potential changes

            if (net_data != NULL && net_data->is_synced) {
                if (actor->id == 0) {
                    actor = next_actor;
                    continue; // Skip local player, continue with next actor
                }

                // This is a remote actor - check if we have data for it
                for (u32 j = 0; j < player_count; j++) {
                    const char* player_id = &ids_buffer[j * 64];

                    if (strcmp(net_data->player_id, player_id) == 0) {
                        // Found a match, update actor with remote data
                        if (NetworkPlayGetRemotePlayerData(player_id, &remote_data)) {
                            Math_Vec3s_Copy(&actor->shape.rot, &remote_data.shapeRotation);

                            // let's offset the position by just a bit on the x, since we're currently tracking ourselves
                            Math_Vec3f_Copy(&actor->world.pos, &remote_data.worldPosition);

                            // For player actors, update more data
                            Player* player = (Player*)actor; // hard coded to assume player, should be generalized later.
                            player->currentBoots = remote_data.currentBoots;
                            player->currentShield = remote_data.currentShield;

                            // Copy joint table if needed
                            for (int k = 0; k < 24; k++) {
                                Math_Vec3s_Copy(&player->skelAnime.jointTable[k], &remote_data.jointTable[k]);
                            }

                            Math_Vec3s_Copy(&player->upperLimbRot, &remote_data.upperLimbRot);

                            break;
                        }
                    }
                }
            }

            actor = next_actor;
        }
    }
}

// MARK: - API

RECOMP_EXPORT void NP_Init() {
    NetworkPlayInit();

    // Create actor extension for network player data
    if (gNetworkPlayerExtension == ACTOR_EXTENSION_INVALID) {
        gNetworkPlayerExtension = z64recomp_extend_actor_all(sizeof(NetworkPlayerData));
        if (gNetworkPlayerExtension == ACTOR_EXTENSION_INVALID) {
            recomp_printf("Failed to create network player extension\n");
        }
    }

    // Reset synced categories
    for (int i = 0; i < MAX_ACTOR_CATEGORIES; i++) {
        gSyncedActorCategories[i] = 0;
    }
}

RECOMP_EXPORT u8 NP_Connect(const char* host) {
    return NetworkPlayConnect(host);
}

RECOMP_EXPORT u8 NP_JoinSession(const char* session) {
    return NetworkPlayJoinSession(session);
}

RECOMP_EXPORT u8 NP_LeaveSession() {
    return NetworkPlayLeaveSession();
}

RECOMP_EXPORT const char* NP_GetActorNetworkId(Actor *actor) {
    if (actor == NULL) {
        recomp_printf("Cannot get ID for NULL actor\n");
        return NULL;
    }

    // Get the network data for this actor
    NetworkPlayerData* netData = GetActorNetworkData(actor);
    if (netData == NULL) {
        recomp_printf("Actor %u is not registered for network play\n", actor->id);
        return NULL;
    }

    // Check if we have a valid player ID
    if (netData->player_id[0] == '\0') {
        return NULL;
    }

    // Return the player ID string
    return netData->player_id;
}

// MARK: - Syncing

RECOMP_EXPORT void NP_SyncActor(Actor* actor, const char* playerID) {
    if (actor == NULL) {
        recomp_printf("Cannot sync NULL actor\n");
        return;
    }

    // Extension creation should be handled in NP_Init, but just in case someone calls this first
    if (gNetworkPlayerExtension == ACTOR_EXTENSION_INVALID) {
        gNetworkPlayerExtension = z64recomp_extend_actor_all(sizeof(NetworkPlayerData));
    }

    // Get or create network data for this actor
    NetworkPlayerData* netData = GetActorNetworkData(actor);
    if (netData == NULL) {
        recomp_printf("Failed to get network data for actor %u\n", actor->id);
        return;
    }

    // Mark actor as synced
    netData->is_synced = 1;

    // Mark this category as containing synced actors
    if (actor->category < MAX_ACTOR_CATEGORIES) {
        gSyncedActorCategories[actor->category] = 1;
    }

    if (actor->id == 0) {
        char playerIdBuffer[64];
        u8 success = NetworkPlayGetPlayerId(playerIdBuffer, sizeof(playerIdBuffer));
        if (success) {
            strcpy(netData->player_id, playerIdBuffer);
            recomp_printf("Added player to sync system\n");
        } else {
            recomp_printf("Failed to get player ID\n");
        }
    } else {
        strcpy(netData->player_id, playerID);
        recomp_printf("Added actor %u to sync system\n", actor->id);
    }
}

RECOMP_EXPORT u32 NP_GetRemotePlayerIDs(u32 maxPlayers, char* idsBuffer, u32 idBufferSize) {
    return NetworkPlayGetRemotePlayerIDs(maxPlayers, idsBuffer, idBufferSize);
}

RECOMP_EXPORT u32 NP_GetRemotePlayerData(const char *playerID, void* dataBuffer) {
    return NetworkPlayGetRemotePlayerData(playerID, dataBuffer);
}
