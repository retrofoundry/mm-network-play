#ifndef MESSAGE_SYSTEM_H
#define MESSAGE_SYSTEM_H

#include "global.h"

// MARK: - Message System API

u8 MessageSystemRegisterHandler(const char* messageId, u32 payloadSize, void* callback);
u8 MessageSystemEmit(const char* messageId, void* data);
void MessageSystemProcessPending();

#endif // MESSAGE_SYSTEM_H 