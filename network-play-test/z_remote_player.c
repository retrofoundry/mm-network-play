#include "z_remote_player.h"

#define FLAGS ACTOR_FLAG_FRIENDLY

void RemotePlayer_Init(Actor* thisx, PlayState* play);
void RemotePlayer_Destroy(Actor* thisx, PlayState* play);
void RemotePlayer_Update(Actor* thisx, PlayState* play);
void RemotePlayer_Draw(Actor* thisx, PlayState* play);

ActorProfile RemotePlayer_InitVars = {
    /**/ ACTOR_ID_MAX,
    /**/ ACTORCAT_NPC,
    /**/ FLAGS,
    /**/ GAMEPLAY_KEEP,
    /**/ sizeof(RemotePlayer),
    /**/ RemotePlayer_Init,
    /**/ RemotePlayer_Destroy,
    /**/ RemotePlayer_Update,
    /**/ RemotePlayer_Draw,
};

void RemotePlayer_Init(Actor* thisx, PlayState* play) {}
void RemotePlayer_Destroy(Actor* thisx, PlayState* play) {}
void RemotePlayer_Update(Actor* thisx, PlayState* play) {}
void RemotePlayer_Draw(Actor* thisx, PlayState* play) {}
