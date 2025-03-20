#include <stdint.h>
#include <string.h>

#include "modding.h"
#include "global.h"
#include "recomputils.h"
#include "network_core.h"

// Message callback registry
typedef struct {
    char message_id[64];          // String identifier for the message
    u32 payload_size;             // Expected payload size
    void (*callback)(void* data); // Callback function
} MessageCallback;

// Maximum number of message handlers we can register
#define MAX_MESSAGE_CALLBACKS 32

// Array to store registered message handlers
static MessageCallback gMessageCallbacks[MAX_MESSAGE_CALLBACKS];
static u32 gMessageCallbackCount = 0;

// MARK: - Message System Implementation

u8 MessageSystemRegisterHandler(const char* messageId, u32 payloadSize, void* callback) {
    if (gMessageCallbackCount >= MAX_MESSAGE_CALLBACKS) {
        recomp_printf("Error: Maximum number of message handlers reached\n");
        return 1;
    }

    if (callback == NULL) {
        recomp_printf("Error: Callback cannot be NULL\n");
        return 1;
    }

    // Check if message ID is already registered
    for (u32 i = 0; i < gMessageCallbackCount; i++) {
        if (strcmp(gMessageCallbacks[i].message_id, messageId) == 0) {
            // Just update the existing handler
            gMessageCallbacks[i].payload_size = payloadSize;
            gMessageCallbacks[i].callback = callback;
            recomp_printf("Updated message handler for '%s'\n", messageId);
            return 0;
        }
    }

    // Register new handler
    strcpy(gMessageCallbacks[gMessageCallbackCount].message_id, messageId);
    gMessageCallbacks[gMessageCallbackCount].message_id[sizeof(gMessageCallbacks[0].message_id) - 1] = '\0'; // Ensure null termination
    gMessageCallbacks[gMessageCallbackCount].payload_size = payloadSize;
    gMessageCallbacks[gMessageCallbackCount].callback = callback;
    gMessageCallbackCount++;

    recomp_printf("Registered message handler for '%s' with payload size %u\n", messageId, payloadSize);
    return 0;
}

u8 MessageSystemEmit(const char* messageId, void* data) {
    // Find the registered size for this message type
    u32 size = 0;
    for (u32 i = 0; i < gMessageCallbackCount; i++) {
        if (strcmp(gMessageCallbacks[i].message_id, messageId) == 0) {
            size = gMessageCallbacks[i].payload_size;
            break;
        }
    }

    if (size == 0) {
        recomp_printf("Warning: Emitting unregistered message type '%s'\n", messageId);
        return 1;
    }

    // Send the message to the server for broadcasting
    return NetworkSyncEmitMessage(messageId, size, data);
}

void MessageSystemProcessPending() {
    while (true) {
        u32 messageSize = NetworkSyncGetPendingMessageSize();
        if (messageSize == 0) {
            break; // No more messages
        }

        // Allocate buffer for the message
        void* buffer = recomp_alloc(messageSize + sizeof(u32));
        char messageId[64] = {0};

        // Get the message and its ID
        u8 success = NetworkSyncGetMessage(buffer, messageSize, messageId);

        // Find the callback for this message type
        for (u32 i = 0; i < gMessageCallbackCount; i++) {
            if (strcmp(gMessageCallbacks[i].message_id, messageId) == 0) {
                gMessageCallbacks[i].callback(buffer);
                break;
            }
        }

        // Free the buffer
        recomp_free(buffer);
    }
} 