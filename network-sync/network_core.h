#ifndef NETWORK_CORE_H
#define NETWORK_CORE_H

#include "modding.h"
#include <stdint.h>

// MARK: - Network Core Imports

RECOMP_IMPORT(".", void NetworkSyncInit());
RECOMP_IMPORT(".", u8 NetworkSyncConnect(const char* host));
RECOMP_IMPORT(".", u8 NetworkSyncJoinSession(const char* session));
RECOMP_IMPORT(".", u8 NetworkSyncLeaveSession());
RECOMP_IMPORT(".", u8 NetworkSyncGetClientId(char* buffer, u32 bufferSize));
RECOMP_IMPORT(".", void NetworkSyncEmitActorData(void* data));
RECOMP_IMPORT(".", u32 NetworkSyncGetRemoteActorIDs(u32 maxPlayers, char* idsBuffer, u32 idBufferSize));
RECOMP_IMPORT(".", u32 NetworkSyncGetRemoteActorData(const char* actor_id, void* dataBuffer));
RECOMP_IMPORT(".", u8 NetworkSyncEmitMessage(const char* messageId, u32 size, void* data));
RECOMP_IMPORT(".", u32 NetworkSyncGetPendingMessageSize());
RECOMP_IMPORT(".", u8 NetworkSyncGetMessage(void* buffer, u32 bufferSize, char* messageIdBuffer));

#endif // NETWORK_CORE_H 