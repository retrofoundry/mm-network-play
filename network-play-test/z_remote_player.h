#ifndef Z_REMOTE_PLAYER_H
#define Z_REMOTE_PLAYER_H

#include "modding.h"
#include "global.h"
#include "recomputils.h"
#include "zelda_arena.h"
#include "z64recomp_api.h"

extern PlayerAgeProperties sPlayerAgeProperties[PLAYER_FORM_MAX];
extern ActorExtensionId REMOTE_PLAYER_ID_EXT;

PlayerAnimationHeader* Player_GetIdleAnim(Player* this);
void Player_Anim_PlayOnceMorph(PlayState* play, Player* this, PlayerAnimationHeader* anim);
PlayerAnimationHeader* func_8082ED20(Player* this);
void Player_DrawGameplay(PlayState* play, Player* this, s32 lod, Gfx* cullDList, OverrideLimbDrawFlex overrideLimbDraw);

#endif
