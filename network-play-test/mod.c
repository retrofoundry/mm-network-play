#include "modding.h"
#include "global.h"

RECOMP_IMPORT("mm_network_play", void NP_Init());
RECOMP_IMPORT("*", int recomp_printf(const char* fmt, ...));

RECOMP_CALLBACK("*", recomp_on_init)
void init_runtime() {
    recomp_printf("Initializing network play...");
    NP_Init();
}
