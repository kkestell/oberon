#include <math.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <gc.h>

void oberon_init(void)              { GC_INIT(); }
void *oberon_alloc(size_t n)        { return GC_MALLOC(n); }
void *oberon_alloc_atomic(size_t n) { return GC_MALLOC_ATOMIC(n); }

/* Report 8.1: an index must lie between zero and the length less one. The
   check runs before the element address is formed, so a zero-length array
   rejects every index and no invalid address is ever computed. */
void oberon_check_index(int32_t index, int32_t length)
{
    if (index < 0 || index >= length) {
        fputs("array index out of bounds\n", stderr);
        exit(1);
    }
}

/* Report 9.1: an array assignment copies the value. memmove rather than memcpy
   because assigning a variable to itself is legal and has to mean something.
   A count of zero is the ordinary result of copying a zero-length array. */
void oberon_copy(void *destination, const void *source, size_t count)
{
    memmove(destination, source, count);
}

/* Oakwood Out.Int(i, n): right-justified in a field of at least n spaces. */
void oberon_out_int(int32_t v, int32_t n) { printf("%*d", (int)n, (int)v); }
void oberon_out_ln(void)                  { putchar('\n'); }

/* Report 10.2 REAL operations. REAL is IEEE 754 binary32, so every one of
   these is the C float form of the same operation. */

float oberon_abs_real(float x) { return fabsf(x); }

/* FLOOR yields the largest INTEGER not greater than x, so a result must exist
   in the signed 32-bit range. The domain is checked here rather than left to
   the float-to-integer conversion, which no target has to define. Both
   endpoints are exact binary32 values. */
int32_t oberon_floor(float x)
{
    if (!isfinite(x) || x < -2147483648.0f || x >= 2147483648.0f) {
        fputs("FLOOR result is outside INTEGER range\n", stderr);
        exit(1);
    }
    return (int32_t)floorf(x);
}

void oberon_pack(float *x, int32_t n) { *x = ldexpf(*x, (int)n); }

/* Report 10.2 normalizes the magnitude to [1, 2), while frexpf normalizes it
   to [0.5, 1), so the fraction doubles and the exponent drops by one. Zero
   satisfies no normalization interval; storing positive zero and a zero
   exponent round-trips through PACK and needs no invented exponent. */
void oberon_unpk(float *x, int32_t *n)
{
    int exponent;

    if (!isfinite(*x)) {
        fputs("UNPK argument is not finite\n", stderr);
        exit(1);
    }
    if (*x == 0.0f) {
        *x = 0.0f;
        *n = 0;
        return;
    }
    *x = frexpf(*x, &exponent) * 2.0f;
    *n = (int32_t)(exponent - 1);
}

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

void oberon_set_element_range(void)
{
    fputs("SET element out of range\n", stderr);
    exit(1);
}
