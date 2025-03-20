#include <stdint.h>
#include <string.h>

#include "modding.h"
#include "global.h"
#include "recomputils.h"
#include "z64recomp_api.h"
#include "network_core.h"
#include "message_system.h"

// MARK: - Actor Extension

// Extension ID for network player data
const ActorExtensionId ACTOR_EXTENSION_INVALID = UINT32_MAX;
static ActorExtensionId gNetworkSyncerExtension = ACTOR_EXTENSION_INVALID;

#define MAX_ACTOR_CATEGORIES 12
const u32 MAX_SYNCED_ACTORS = 32;
static u8 gSyncedActorCategories[MAX_ACTOR_CATEGORIES] = {0};  // Bitset for categories with synced actors

// Structure to hold network-specific data for each actor
typedef struct {
    // UUID string for this actor
    char actor_id[64];
    // Flag indicating if actor is being synced
    u8 is_synced;
    // Flag indicating whether we are in charge of pushing its data to the server
    u8 is_owned_locally;
} NetworkExtendedActorData;

static NetworkExtendedActorData* GetActorNetworkData(Actor* actor) {
    if (gNetworkSyncerExtension == ACTOR_EXTENSION_INVALID) {
        return NULL;
    }

    return (NetworkExtendedActorData*)z64recomp_get_extended_actor_data(actor, gNetworkSyncerExtension);
}

// MARK: - Actor Sync Data

typedef struct {
    Vec3f worldPosition;
    Vec3s shapeRotation;

    // Player Actor specific properties
    Vec3s upperLimbRot;
    Vec3s jointTable[24];
    s8 currentMask;
    s8 currentShield;
} ActorSyncData;

// MARK: - Actor Sync Implementation

void ActorSyncInit() {
    // Create actor extension for network player data
    if (gNetworkSyncerExtension == ACTOR_EXTENSION_INVALID) {
        gNetworkSyncerExtension = z64recomp_extend_actor_all(sizeof(NetworkExtendedActorData));
        if (gNetworkSyncerExtension == ACTOR_EXTENSION_INVALID) {
            recomp_printf("Failed to create network player extension\n");
        }
    }

    // Reset synced categories
    for (int i = 0; i < MAX_ACTOR_CATEGORIES; i++) {
        gSyncedActorCategories[i] = 0;
    }
}

const char* ActorSyncGetNetworkId(Actor *actor) {
    if (actor == NULL) {
        recomp_printf("Cannot get ID for NULL actor\n");
        return NULL;
    }

    NetworkExtendedActorData* netData = GetActorNetworkData(actor);
    if (netData == NULL) {
        recomp_printf("Actor %u is not registered for network play\n", actor->id);
        return NULL;
    }

    if (netData->actor_id[0] == '\0') {
        return NULL;
    }

    return netData->actor_id;
}

void ActorSyncRegister(Actor* actor, const char* playerId, int isOwnedLocally) {
    if (actor == NULL) {
        recomp_printf("Cannot sync NULL actor\n");
        return;
    }

    if (gNetworkSyncerExtension == ACTOR_EXTENSION_INVALID) {
        gNetworkSyncerExtension = z64recomp_extend_actor_all(sizeof(NetworkExtendedActorData));
    }

    NetworkExtendedActorData* netData = GetActorNetworkData(actor);
    if (netData == NULL) {
        recomp_printf("Failed to get network data for actor %u\n", actor->id);
        return;
    }

    netData->is_synced = 1;
    netData->is_owned_locally = isOwnedLocally;

    if (actor->category < MAX_ACTOR_CATEGORIES) {
        gSyncedActorCategories[actor->category] = 1;
    }

    if (actor->id == 0) {
        char playerIdBuffer[64];
        u8 success = NetworkSyncGetClientId(playerIdBuffer, sizeof(playerIdBuffer));
        if (success) {
            strcpy(netData->actor_id, playerIdBuffer);
            recomp_printf("Added player to sync system\n");
        } else {
            recomp_printf("Failed to get player ID\n");
        }
    } else if (playerId != NULL) {
        strcpy(netData->actor_id, playerId);
    }
}

void ActorSyncUpdate(PlayState* play, Actor* actor) {
    NetworkExtendedActorData* netData = GetActorNetworkData(actor);

    if (netData == NULL || !netData->is_synced || !netData->is_owned_locally) {
        return;
    }

    ActorSyncData* syncData = recomp_alloc(sizeof(ActorSyncData) + sizeof(Vec3s) * 23);
    Math_Vec3s_Copy(&syncData->shapeRotation, &actor->shape.rot);
    Math_Vec3f_Copy(&syncData->worldPosition, &actor->world.pos);

    if (actor->category == ACTORCAT_PLAYER) {
        Player* player = (Player*)actor;
        syncData->currentMask = player->currentMask;
        syncData->currentShield = player->currentShield;

        for (int i = 0; i < 24; i++) {
            Math_Vec3s_Copy(&syncData->jointTable[i], &player->skelAnime.jointTable[i]);
        }

        Math_Vec3s_Copy(&syncData->upperLimbRot, &player->upperLimbRot);
    }

    NetworkSyncEmitActorData(syncData);
    recomp_free(syncData);
}

void ActorSyncProcessRemoteData(PlayState* play) {
    ActorSyncData remote_data;
    char ids_buffer[MAX_SYNCED_ACTORS * 64];
    u32 player_count = NetworkSyncGetRemoteActorIDs(MAX_SYNCED_ACTORS, ids_buffer, 64);

    for (u32 i = 0; i < MAX_ACTOR_CATEGORIES; i++) {
        if (gSyncedActorCategories[i] == 0) {
            continue;
        }

        Actor* actor = play->actorCtx.actorLists[i].first;

        while (actor != NULL) {
            NetworkExtendedActorData* net_data = GetActorNetworkData(actor);
            Actor* next_actor = actor->next;

            if (net_data != NULL && net_data->is_synced) {
                if (net_data->is_owned_locally) {
                    actor = next_actor;
                    continue;
                }

                for (u32 j = 0; j < player_count; j++) {
                    const char* actor_id = &ids_buffer[j * 64];

                    if (strcmp(net_data->actor_id, actor_id) == 0) {
                        if (NetworkSyncGetRemoteActorData(actor_id, &remote_data)) {
                            Math_Vec3s_Copy(&actor->shape.rot, &remote_data.shapeRotation);
                            Math_Vec3f_Copy(&actor->world.pos, &remote_data.worldPosition);

                            if (actor->category == ACTORCAT_PLAYER) {
                                Player* player = (Player*)actor;
                                player->currentMask = remote_data.currentMask;
                                player->currentShield = remote_data.currentShield;

                                for (int k = 0; k < 24; k++) {
                                    Math_Vec3s_Copy(&player->skelAnime.jointTable[k], &remote_data.jointTable[k]);
                                }

                                Math_Vec3s_Copy(&player->upperLimbRot, &remote_data.upperLimbRot);
                            }

                            break;
                        }
                    }
                }
            }

            actor = next_actor;
        }
    }
} 