#include "modding.h"
#include "global.h"

RECOMP_IMPORT(".", void NetworkPlayInit());
RECOMP_IMPORT(".", u8 NetworkPlayConnect(const char* host));
RECOMP_IMPORT(".", void NetworkPlaySetPlayerId(u32 id));
RECOMP_IMPORT(".", u8 NetworkPlaySetPlayerCanSpin(u32 canSpin));
RECOMP_IMPORT(".", u8 NetworkPlayCanPlayerSpin(u32 playerId));
RECOMP_IMPORT("*", int recomp_printf(const char* fmt, ...));

RECOMP_CALLBACK("*", recomp_on_init)
void init_runtime() {
    NetworkPlayInit();
    NetworkPlayConnect("wss://echo.websocket.org");

    // Setters
    NetworkPlaySetPlayerId(1);
    NetworkPlaySetPlayerCanSpin(1);
}


// Patches a function in the base game that's used to check if the player should quickspin.
RECOMP_PATCH s32 Player_CanSpinAttack(Player* this) {
    recomp_printf("Spin attacking\n");
    return NetworkPlayCanPlayerSpin(1);
}
