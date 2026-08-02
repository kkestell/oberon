#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#include <gc.h>

void oberon_init(void)              { GC_INIT(); }
void *oberon_alloc(size_t n)        { return GC_MALLOC(n); }
void *oberon_alloc_atomic(size_t n) { return GC_MALLOC_ATOMIC(n); }

/* Oakwood Out.Int(i, n): right-justified in a field of at least n spaces. */
void oberon_out_int(int32_t v, int32_t n) { printf("%*d", (int)n, (int)v); }
void oberon_out_ln(void)                  { putchar('\n'); }
