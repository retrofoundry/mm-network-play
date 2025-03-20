#ifndef CALLBACKS_H
#define CALLBACKS_H

#include "modding.h"
#include "global.h"
#include "z64recomp_api.h"

// MARK: - Game Event Callbacks

void on_actor_update(PlayState* play, Actor* actor);
void on_play_main(PlayState* play);

#endif // CALLBACKS_H 