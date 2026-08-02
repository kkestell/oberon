PREFIX ?= $(HOME)/.local
CFLAGS = -O2

# The compiler looks for its support directory at ../lib/oberon from its own
# executable, so target/debug/oberon and target/release/oberon share this one.
SUPPORT = target/lib/oberon
OBJ = target/obj

STAGED = $(SUPPORT)/qbe $(SUPPORT)/liboberon.a
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

$(OBJ)/%.o: runtime/%.c
	mkdir -p $(OBJ)
	$(CC) $(CFLAGS) -c $< -o $@

$(SUPPORT)/liboberon.a: $(OBJ)/oberon.o
	mkdir -p $(SUPPORT)
	rm -f $@
	ar rcs $@ $^

.PHONY: all compiler test install uninstall clean
