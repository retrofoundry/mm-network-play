BUILD_DIR := build
MAIN_MOD_NAME := mm_network_sync
TEST_MOD_NAME := mm_network_sync_test
DYLIB_DIR := network-sync-runtime
DYLIB_BASE_NAME := network_sync_runtime
SKIP_RUST ?= 0
DEBUG ?= 1

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
SYMS_PATH := deps/Zelda64RecompSyms/mm.us.rev1.syms.toml

# Set build type based on DEBUG flag
ifeq ($(DEBUG),1)
    CPPFLAGS_EXTRA := -D_DEBUG
    CARGO_PROFILE := debug
else
    CPPFLAGS_EXTRA := -DNDEBUG
    CARGO_PROFILE := release
endif

# Main mod targets
MAIN_TARGET  := $(BUILD_DIR)/main/mod.elf
MAIN_NRM_TARGET := $(BUILD_DIR)/$(MAIN_MOD_NAME).nrm
DYLIB_TARGET := $(BUILD_DIR)/$(DYLIB_TARGET_NAME)

# Test mod targets
TEST_TARGET  := $(BUILD_DIR)/test/mod.elf
TEST_NRM_TARGET := $(BUILD_DIR)/$(TEST_MOD_NAME).nrm

# Actual outputs
MAIN_NRM_ACTUAL := $(BUILD_DIR)/main/$(MAIN_MOD_NAME).nrm
TEST_NRM_ACTUAL := $(BUILD_DIR)/test/$(TEST_MOD_NAME).nrm

ifeq ($(OS),Windows_NT)
	INSTALL_DIR := $(LOCALAPPDATA)/Zelda64Recompiled/mods
else
	INSTALL_DIR := $(HOME)/.config/Zelda64Recompiled/mods
endif

LDSCRIPT := mod.ld
CFLAGS   := -target mips -mips2 -mabi=32 -O2 -G0 -mno-abicalls -mno-odd-spreg -mno-check-zero-division \
            -fomit-frame-pointer -ffast-math -fno-unsafe-math-optimizations -fno-builtin-memset \
            -Wall -Wextra -Wno-incompatible-library-redeclaration -Wno-unused-parameter -Wno-unknown-pragmas -Wno-unused-variable \
            -Wno-missing-braces -Wno-unsupported-floating-point-opt -Werror=section
CPPFLAGS := -nostdinc -D_LANGUAGE_C -DMIPS -DF3DEX_GBI_2 -DF3DEX_GBI_PL -DGBI_DOWHILE $(CPPFLAGS_EXTRA) \
            -I include -I include/dummy_headers \
            -I deps/mm-decomp/include -I deps/mm-decomp/src -I deps/mm-decomp/extracted/n64-us -I deps/mm-decomp/include/libc
LDFLAGS  := -nostdlib -T $(LDSCRIPT) -Map $(BUILD_DIR)/mod.map --unresolved-symbols=ignore-all --emit-relocs -e 0 --no-nmagic

# Main mod files
MAIN_C_SRCS := $(wildcard network-sync/*.c)
MAIN_C_OBJS := $(addprefix $(BUILD_DIR)/main/, $(MAIN_C_SRCS:.c=.o))
MAIN_C_DEPS := $(addprefix $(BUILD_DIR)/main/, $(MAIN_C_SRCS:.c=.d))

# Test mod files
TEST_C_SRCS := $(wildcard network-sync-test/*.c network-sync-test/**/*.c)
TEST_C_OBJS := $(addprefix $(BUILD_DIR)/test/, $(TEST_C_SRCS:.c=.o))
TEST_C_DEPS := $(addprefix $(BUILD_DIR)/test/, $(TEST_C_SRCS:.c=.d))

.PHONY: all clean main test release build-dylib install

all: main test

release:
	$(MAKE) DEBUG=0

debug:
	$(MAKE) DEBUG=1

main: $(MAIN_NRM_TARGET) $(DYLIB_TARGET)

test: $(TEST_NRM_TARGET)

# Install target to copy files to Zelda64Recompiled mods directory
install: main test
	@echo "Installing mod files to $(INSTALL_DIR)"
ifeq ($(OS),Windows_NT)
	@if not exist "$(INSTALL_DIR)" mkdir "$(INSTALL_DIR)"
	copy "$(MAIN_NRM_ACTUAL)" "$(INSTALL_DIR)"
	copy "$(TEST_NRM_ACTUAL)" "$(INSTALL_DIR)"
	copy "$(DYLIB_TARGET)" "$(INSTALL_DIR)"
else
	@mkdir -p "$(INSTALL_DIR)"
	cp "$(MAIN_NRM_ACTUAL)" "$(INSTALL_DIR)"
	cp "$(TEST_NRM_ACTUAL)" "$(INSTALL_DIR)"
	cp "$(DYLIB_TARGET)" "$(INSTALL_DIR)"
endif
	@echo "Installation complete"

# Step 1: Build the main .elf file
$(MAIN_TARGET): $(MAIN_C_OBJS) $(LDSCRIPT) | $(BUILD_DIR)/main
	$(LD) $(MAIN_C_OBJS) $(LDFLAGS) -o $@

# Step 2: Run RecompModTool to generate main .nrm file
$(MAIN_NRM_TARGET): $(MAIN_TARGET) | $(BUILD_DIR)
	$(MOD_TOOL) mod.toml $(BUILD_DIR)/main

# Step 3: Build the Rust dylib
RUST_SRCS := $(shell find $(DYLIB_DIR)/src deps/gamecore/src deps/n64-recomp/src deps/gamecore/src -name "*.rs" 2>/dev/null)
CARGO_TOML := $(DYLIB_DIR)/Cargo.toml

$(DYLIB_TARGET): $(RUST_SRCS) $(CARGO_TOML) | $(BUILD_DIR)
ifeq ($(SKIP_RUST),0)
	cd $(DYLIB_DIR) && cargo build $(if $(filter release,$(CARGO_PROFILE)),--release,)
ifeq ($(OS),Windows_NT)
	copy "$(DYLIB_DIR)\target\$(CARGO_PROFILE)\$(DYLIB_SRC_NAME)" "$(BUILD_DIR)\$(DYLIB_TARGET_NAME)"
else
	cp $(DYLIB_DIR)/target/$(CARGO_PROFILE)/$(DYLIB_SRC_NAME) $(BUILD_DIR)/$(DYLIB_TARGET_NAME)
endif
endif

# Step 4: Build the test .elf file
$(TEST_TARGET): $(TEST_C_OBJS) $(LDSCRIPT) | $(BUILD_DIR)/test
	$(LD) $(TEST_C_OBJS) $(LDFLAGS) -o $@

# Step 5: Run RecompModTool to generate test .nrm file
$(TEST_NRM_TARGET): $(TEST_TARGET) | $(BUILD_DIR)/test
	$(MOD_TOOL) network-sync-test/test.toml $(BUILD_DIR)/test

$(BUILD_DIR) $(BUILD_DIR)/main $(BUILD_DIR)/main/network-sync $(BUILD_DIR)/test $(BUILD_DIR)/test/network-sync-test:
ifeq ($(OS),Windows_NT)
	mkdir $(subst /,\,$@)
else
	mkdir -p $@
endif

$(MAIN_C_OBJS): $(BUILD_DIR)/main/%.o : %.c | $(BUILD_DIR) $(BUILD_DIR)/main $(BUILD_DIR)/main/network-sync
	$(CC) $(CFLAGS) $(CPPFLAGS) $< -MMD -MF $(@:.o=.d) -c -o $@

$(TEST_C_OBJS): $(BUILD_DIR)/test/%.o : %.c | $(BUILD_DIR) $(BUILD_DIR)/test $(BUILD_DIR)/test/network-sync-test
	$(CC) $(CFLAGS) $(CPPFLAGS) $< -MMD -MF $(@:.o=.d) -c -o $@

clean:
	rm -rf $(BUILD_DIR)
	cd $(DYLIB_DIR) && cargo clean

-include $(MAIN_C_DEPS)
-include $(TEST_C_DEPS)
