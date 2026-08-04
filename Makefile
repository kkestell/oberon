PREFIX ?= $(HOME)/.local
CFLAGS = -O2 -Ivendor/bdwgc/include

# The compiler looks for its support directory at ../lib/oberon from its own
# executable, so target/debug/oberon and target/release/oberon share this one.
SUPPORT = target/lib/oberon
OBJ = target/obj

MODULES = $(patsubst lib/%,$(SUPPORT)/%,$(wildcard lib/*.Mod))
STAGED = $(SUPPORT)/qbe $(SUPPORT)/liboberon.a $(MODULES)
GC_SOURCES = $(wildcard vendor/bdwgc/*.c vendor/bdwgc/extra/*.c vendor/bdwgc/include/*.h vendor/bdwgc/include/private/*.h)
QBE_SOURCES = $(wildcard vendor/qbe/*.c vendor/qbe/*.h vendor/qbe/*/*.c vendor/qbe/*/*.h)

all: compiler $(STAGED)

compiler:
	cargo build --release

test: $(STAGED)
	cargo test

install: all
	mkdir -p $(PREFIX)/bin
	cp target/release/oberon $(PREFIX)/bin/oberon
	rm -rf $(PREFIX)/lib/oberon
	mkdir -p $(PREFIX)/lib
	cp -R $(SUPPORT) $(PREFIX)/lib/oberon

uninstall:
	rm -f $(PREFIX)/bin/oberon
	rm -rf $(PREFIX)/lib/oberon

clean:
	cargo clean
	$(MAKE) -C vendor/qbe clean-gen

$(SUPPORT)/qbe: $(QBE_SOURCES)
	$(MAKE) -C vendor/qbe qbe
	mkdir -p $(SUPPORT)
	cp vendor/qbe/qbe $@

# Generated programs are single-threaded, so the collector is built without
# thread support, as the one translation unit its sources provide for this.
# Its own builds define NO_EXECUTE_PERMISSION by default; without it the heap
# is mapped executable, which Apple ARM64 refuses.
$(OBJ)/gc.o: $(GC_SOURCES)
	mkdir -p $(OBJ)
	$(CC) $(CFLAGS) -DNO_EXECUTE_PERMISSION -c vendor/bdwgc/extra/gc.c -o $@

$(OBJ)/%.o: runtime/%.c
	mkdir -p $(OBJ)
	$(CC) $(CFLAGS) -c $< -o $@

$(SUPPORT)/liboberon.a: $(OBJ)/gc.o $(OBJ)/oberon.o $(OBJ)/standard.o
	mkdir -p $(SUPPORT)
	rm -f $@
	ar rcs $@ $^

$(SUPPORT)/%.Mod: lib/%.Mod
	mkdir -p $(SUPPORT)
	cp $< $@

.PHONY: all compiler test install uninstall clean
