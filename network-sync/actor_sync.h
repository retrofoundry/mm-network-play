#ifndef ACTOR_SYNC_H
#define ACTOR_SYNC_H

#include "global.h"

// MARK: - Actor Sync API

void ActorSyncInit();
const char* ActorSyncGetNetworkId(Actor *actor);
void ActorSyncRegister(Actor* actor, const char* playerId, int isOwnedLocally);

// MARK: - Internal API (used by callbacks)
void ActorSyncUpdate(PlayState* play, Actor* actor);
void ActorSyncProcessRemoteData(PlayState* play);

#endif // ACTOR_SYNC_H 