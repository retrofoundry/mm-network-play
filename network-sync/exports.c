#include "modding.h"
#include "global.h"
#include "network_core.h"
#include "actor_sync.h"
#include "message_system.h"

// MARK: - Core Network API

RECOMP_EXPORT void NS_Init() {
    NetworkSyncInit();
    ActorSyncInit();
}

RECOMP_EXPORT u8 NS_Connect(const char* host) {
    return NetworkSyncConnect(host);
}

RECOMP_EXPORT u8 NS_JoinSession(const char* session) {
    return NetworkSyncJoinSession(session);
}

RECOMP_EXPORT u8 NS_LeaveSession() {
    return NetworkSyncLeaveSession();
}

// MARK: - Actor Sync API

RECOMP_EXPORT const char* NS_GetActorNetworkId(Actor *actor) {
    return ActorSyncGetNetworkId(actor);
}

RECOMP_EXPORT void NS_SyncActor(Actor* actor, const char* playerId, int isOwnedLocally) {
    ActorSyncRegister(actor, playerId, isOwnedLocally);
}

RECOMP_EXPORT u32 NS_GetRemoteActorIDs(u32 maxPlayers, char* idsBuffer, u32 idBufferSize) {
    return NetworkSyncGetRemoteActorIDs(maxPlayers, idsBuffer, idBufferSize);
}

RECOMP_EXPORT u32 NS_GetRemoteActorData(const char *playerID, void* dataBuffer) {
    return NetworkSyncGetRemoteActorData(playerID, dataBuffer);
}

// MARK: - Message System API

RECOMP_EXPORT u8 NS_RegisterMessageHandler(const char* messageId, u32 payloadSize, void* callback) {
    return MessageSystemRegisterHandler(messageId, payloadSize, callback);
}

RECOMP_EXPORT u8 NS_EmitMessage(const char* messageId, void* data) {
    return MessageSystemEmit(messageId, data);
}
