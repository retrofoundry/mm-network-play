#include "z_remote_player.h"

#define FLAGS                                                                                  \
    (ACTOR_FLAG_ATTENTION_ENABLED | ACTOR_FLAG_FRIENDLY | ACTOR_FLAG_UPDATE_CULLING_DISABLED | \
     ACTOR_FLAG_DRAW_CULLING_DISABLED | ACTOR_FLAG_UPDATE_DURING_SOARING_AND_SOT_CS |          \
     ACTOR_FLAG_UPDATE_DURING_OCARINA | ACTOR_FLAG_CAN_PRESS_SWITCHES | ACTOR_FLAG_MINIMAP_ICON_ENABLED)

void RemotePlayer_Init(Actor* thisx, PlayState* play);
void RemotePlayer_Destroy(Actor* thisx, PlayState* play);
void RemotePlayer_Update(Actor* thisx, PlayState* play);
void RemotePlayer_Draw(Actor* thisx, PlayState* play);

ActorProfile RemotePlayer_InitVars = {
    /**/ ACTOR_ID_MAX,
    /**/ ACTORCAT_PLAYER,
    /**/ FLAGS,
    /**/ OBJECT_LINK_CHILD,
    /**/ sizeof(Player),
    /**/ RemotePlayer_Init,
    /**/ RemotePlayer_Destroy,
    /**/ RemotePlayer_Update,
    /**/ RemotePlayer_Draw,
};

void RemotePlayer_Init(Actor* thisx, PlayState* play) {
    Player* player = (Player*)thisx;

        // Primarily modeled after EnTest3_Init and Player_Init
        player->csId = CS_ID_NONE;
        player->transformation = PLAYER_FORM_HUMAN;
        player->ageProperties = &sPlayerAgeProperties[player->transformation];
        player->heldItemAction = PLAYER_IA_NONE;
        player->heldItemId = ITEM_OCARINA_OF_TIME;

        Player_SetModelGroup(player, PLAYER_MODELGROUP_DEFAULT);
        play->playerInit(player, play, gPlayerSkeletons[player->transformation]);

        player->maskObjectSegment = ZeldaArena_Malloc(0x3800);
        // play->func_18780(player, play);
        Player_Anim_PlayOnceMorph(play, player, Player_GetIdleAnim(player));
        player->yaw = player->actor.shape.rot.y;

        // Will ensure the actor is always updating even when in a separate room than the player
        player->actor.room = -1;
}

void RemotePlayer_Destroy(Actor* thisx, PlayState* play) {}

void RemotePlayer_Update(Actor* thisx, PlayState* play) {
    Player* player = (Player*)thisx;

    player->actor.shape.shadowAlpha = 255;
}

void RemotePlayer_Draw(Actor* thisx, PlayState* play) {
    Player* player = (Player*)thisx;
    Player_DrawGameplay(play, player, 1, gCullBackDList, Player_OverrideLimbDrawGameplayDefault);
}
