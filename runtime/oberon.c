#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include <gc.h>

void oberon_init(void)              { GC_INIT(); }
void *oberon_alloc(size_t n)        { return GC_MALLOC(n); }
void *oberon_alloc_atomic(size_t n) { return GC_MALLOC_ATOMIC(n); }

/* Oakwood Out.Int(i, n): right-justified in a field of at least n spaces. */
void oberon_out_int(int32_t v, int32_t n) { printf("%*d", (int)n, (int)v); }
void oberon_out_ln(void)                  { putchar('\n'); }

/* Traps. None carries a source position: naming the source file needs string
   data the compiler cannot emit yet, so that waits for a later slice. */

void oberon_div_by_zero(void)
{
    fputs("DIV or MOD by zero\n", stderr);
    exit(1);
}

void oberon_assert_failed(void)
{
    fputs("assertion failed\n", stderr);
    exit(1);
}

void oberon_abs_overflow(void)
{
    fputs("ABS overflows INTEGER\n", stderr);
    exit(1);
}

void oberon_case_no_match(void)
{
    fputs("CASE without matching label\n", stderr);
    exit(1);
}

void oberon_shift_range(void)
{
    fputs("shift count out of range\n", stderr);
    exit(1);
}
