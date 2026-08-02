#include <stddef.h>

#include <gc.h>

void oberon_init(void)              { GC_INIT(); }
void *oberon_alloc(size_t n)        { return GC_MALLOC(n); }
void *oberon_alloc_atomic(size_t n) { return GC_MALLOC_ATOMIC(n); }
