BUILD_DIR := build
MOD_NAME := mm_network_play-1.0.0
DYLIB_DIR := network-play-runtime
DYLIB_BASE_NAME := network_play_runtime

# Allow the user to specify the compiler and linker on macOS
# as Apple Clang does not support MIPS architecture
ifeq ($(shell uname),Darwin)
    CC      ?= clang
    LD      ?= ld.lld
    DYLIB_EXT := .dylib
    DYLIB_PREFIX := lib
else ifeq ($(OS),Windows_NT)
    CC      := clang
    LD      := ld.lld
    DYLIB_EXT := .dll
    DYLIB_PREFIX :=
else
    CC      := clang
    LD      := ld.lld
    DYLIB_EXT := .so
    DYLIB_PREFIX := lib
endif

# Source and target names for the dylib
DYLIB_SRC_NAME := $(DYLIB_PREFIX)$(DYLIB_BASE_NAME)$(DYLIB_EXT)
DYLIB_TARGET_NAME := $(DYLIB_BASE_NAME)$(DYLIB_EXT)

MOD_TOOL := ./RecompModTool
SYMS_PATH := Zelda64RecompSyms/mm.us.rev1.syms.toml
TARGET  := $(BUILD_DIR)/mod.elf
NRM_TARGET := $(BUILD_DIR)/$(MOD_NAME).nrm
DYLIB_TARGET := $(BUILD_DIR)/$(DYLIB_TARGET_NAME)

LDSCRIPT := mod.ld
CFLAGS   := -target mips -mips2 -mabi=32 -O2 -G0 -mno-abicalls -mno-odd-spreg -mno-check-zero-division \
			-fomit-frame-pointer -ffast-math -fno-unsafe-math-optimizations -fno-builtin-memset \
			-Wall -Wextra -Wno-incompatible-library-redeclaration -Wno-unused-parameter -Wno-unknown-pragmas -Wno-unused-variable \
			-Wno-missing-braces -Wno-unsupported-floating-point-opt -Werror=section
CPPFLAGS := -nostdinc -D_LANGUAGE_C -DMIPS -DF3DEX_GBI_2 -DF3DEX_GBI_PL -DGBI_DOWHILE -I include -I include/dummy_headers \
			-I mm-decomp/include -I mm-decomp/src -I mm-decomp/extracted/n64-us -I mm-decomp/include/libc
LDFLAGS  := -nostdlib -T $(LDSCRIPT) -Map $(BUILD_DIR)/mod.map --unresolved-symbols=ignore-all --emit-relocs -e 0 --no-nmagic

C_SRCS := $(wildcard src/*.c)
C_OBJS := $(addprefix $(BUILD_DIR)/, $(C_SRCS:.c=.o))
C_DEPS := $(addprefix $(BUILD_DIR)/, $(C_SRCS:.c=.d))

.PHONY: all clean build-dylib

all: $(NRM_TARGET) $(DYLIB_TARGET)

# Step 1: Build the .elf file
$(TARGET): $(C_OBJS) $(LDSCRIPT) | $(BUILD_DIR)
	$(LD) $(C_OBJS) $(LDFLAGS) -o $@

# Step 2: Run RecompModTool to generate .nrm file
$(NRM_TARGET): $(TARGET) | $(BUILD_DIR)
	$(MOD_TOOL) mod.toml $(BUILD_DIR)

# Step 3: Build the Rust dylib
$(DYLIB_TARGET): | $(BUILD_DIR)
	cd $(DYLIB_DIR) && cargo build
ifeq ($(OS),Windows_NT)
	copy "$(DYLIB_DIR)\target\debug\$(DYLIB_SRC_NAME)" "$(BUILD_DIR)\$(DYLIB_TARGET_NAME)"
else
	cp $(DYLIB_DIR)/target/debug/$(DYLIB_SRC_NAME) $(BUILD_DIR)/$(DYLIB_TARGET_NAME)
endif

ifeq ($(OS),Windows_NT)
    # For Windows
    $(BUILD_DIR) $(BUILD_DIR)/src:
	    mkdir $(subst /,\,$@)
else
    # For macOS and Linux
    $(BUILD_DIR) $(BUILD_DIR)/src:
	    mkdir -p $@
endif

$(C_OBJS): $(BUILD_DIR)/%.o : %.c | $(BUILD_DIR) $(BUILD_DIR)/src
	$(CC) $(CFLAGS) $(CPPFLAGS) $< -MMD -MF $(@:.o=.d) -c -o $@

clean:
	rm -rf $(BUILD_DIR)
	cd $(DYLIB_DIR) && cargo clean

-include $(C_DEPS)
