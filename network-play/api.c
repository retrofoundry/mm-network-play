#include "modding.h"
#include "global.h"

RECOMP_IMPORT(".", void NetworkPlayInit());
RECOMP_IMPORT(".", u8 NetworkPlayConnect(const char* host));
RECOMP_IMPORT(".", void NetworkPlaySetPlayerId(u32 id));
RECOMP_IMPORT("*", int recomp_printf(const char* fmt, ...));

RECOMP_EXPORT void NP_Init() {
    NetworkPlayInit();
}

RECOMP_EXPORT u8 NP_Connect(const char* host, u32 id) {
    NetworkPlaySetPlayerId(id);
    return NetworkPlayConnect(host);
}
