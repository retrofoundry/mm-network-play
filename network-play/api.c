#include "modding.h"
#include "global.h"
#include "recomputils.h"
#include "recompconfig.h"
#include "recompui.h"
#include "z64recomp_api.h"

// MARK: - Syncing Whitelist

#define MAX_SYNCED_ACTORS 32
typedef struct {
    Actor* actorPtr;      // Pointer to the actor instance
    u32 syncFlags;       // Which properties to sync for this actor
} SyncedActor;

static SyncedActor syncedActors[MAX_SYNCED_ACTORS];
static int syncedActorCount = 0;

// Helper to check if an actor is in our sync whitelist
static SyncedActor* findSyncedActor(Actor* actor) {
    for (int i = 0; i < syncedActorCount; i++) {
        if (syncedActors[i].actorPtr == actor) {
            return &syncedActors[i];
        }
    }
    return NULL;
}

// MARK: - Struct

typedef struct {
    s8 currentBoots;
    s8 currentShield;
    Vec3s jointTable[24]; // Array of Vec3s
    Vec3s upperLimbRot;
    Vec3s shapeRotation;
    Vec3f worldPosition;
} PlayerSyncData;

// MARK: - Imports

RECOMP_IMPORT(".", void NetworkPlayInit());
RECOMP_IMPORT(".", u8 NetworkPlayConnect(const char* host));
RECOMP_IMPORT(".", u8 NetworkPlayJoinSession(const char* session));
RECOMP_IMPORT(".", u8 NetworkPlayLeaveSession());
RECOMP_IMPORT(".", void NetworkPlaySendPlayerSync(PlayerSyncData* data));

// MARK: - Events

RECOMP_CALLBACK("*", recomp_after_actor_update)
void on_actor_update(PlayState* play, Actor* actor) {
    SyncedActor* synced = findSyncedActor(actor);

    // Check if actor is in our whitelist - process sync
    if (synced != NULL) {
        if (actor->id == 0) {
            Player* player = (Player*)actor;

            // Create player sync data structure - need to allocate enough space
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

            // recomp_printf("Syncing player actor: pos=%f,%f,%f\n",
            //     syncData.worldPosition.x, syncData.worldPosition.y, syncData.worldPosition.z);
        } else {
            // Other actor sync
            // recomp_printf("Syncing actor %u: pos=%f,%f,%f\n",
            //     actor->id, actor->world.pos.x, actor->world.pos.y, actor->world.pos.z);
        }
    }
}

// MARK: - API

RECOMP_EXPORT void NP_Init() {
    NetworkPlayInit();
    syncedActorCount = 0;
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

// MARK: - Syncing

RECOMP_EXPORT void NP_SyncActor(Actor* actor, u32 syncFlags) {
    recomp_printf("Syncing actor %u in room %u\n", actor->id, actor->room);

    if (actor == NULL) {
        recomp_printf("Cannot sync NULL actor\n");
        return;
    }

    // Check if actor is already synced
    SyncedActor* existing = findSyncedActor(actor);
    if (existing != NULL) {
        // Update sync flags for existing actor
        existing->syncFlags = syncFlags;
        recomp_printf("Updated sync flags for actor %u\n", actor->id);
        return;
    }

    // Add to whitelist if there's space
    if (syncedActorCount < MAX_SYNCED_ACTORS) {
        syncedActors[syncedActorCount].actorPtr = actor;
        syncedActors[syncedActorCount].syncFlags = syncFlags;
        syncedActorCount++;

        if (actor->id == 0) {
            recomp_printf("Added player to sync whitelist\n");
        } else {
            recomp_printf("Added actor %u to sync whitelist\n", actor->id);
        }
    } else {
        recomp_printf("Cannot sync more actors: whitelist full\n");
    }
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
