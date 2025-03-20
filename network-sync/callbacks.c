#include <stdint.h>

#include "modding.h"
#include "global.h"
#include "actor_sync.h"
#include "message_system.h"

// MARK: - Game Event Callbacks

RECOMP_CALLBACK("*", recomp_after_actor_update)
void on_actor_update(PlayState* play, Actor* actor) {
    // Update actor synchronization
    ActorSyncUpdate(play, actor);
}

RECOMP_CALLBACK("*", recomp_on_play_main)
void on_play_main(PlayState* play) {
    // Process remote actor data
    ActorSyncProcessRemoteData(play);

    // Process pending messages
    MessageSystemProcessPending();
}
