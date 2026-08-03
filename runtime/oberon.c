#include <math.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Every language failure ends here. Out writes through the buffered C stdout,
   so it is flushed before the message goes to the unbuffered stderr: on a
   shared terminal the failure then follows the output the program wrote
   before it. The position is Module:line:column, the module standing in for
   the source path the executable does not know, with the 1-based line and
   column compile diagnostics use. exit flushes every other stream. */
static void oberon_fail(const char *message, const char *module, int32_t line, int32_t col)
{
    fflush(stdout);
    fprintf(stderr, "%s:%d:%d: %s\n", module, (int)line, (int)col, message);
    exit(1);
}

/* Report 8.1: an index must lie between zero and the length less one. The
   check runs before the element address is formed, so a zero-length array
   rejects every index and no invalid address is ever computed. */
int32_t oberon_check_index(
    int32_t index,
    int32_t length,
    const char *module, int32_t line, int32_t col)
{
    if (index < 0 || index >= length) {
        oberon_fail("array index out of bounds", module, line, col);
    }
    return index;
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

/* Out.Char writes the low byte and nothing else, so a program's output is
   exactly the bytes it asked for. */
void oberon_out_char(int32_t c) { putchar((unsigned char)c); }

/* Report 8.2.4's relations on character arrays and strings. Each length
   bounds its operand: a character array's is its declared length and a
   string literal's counts its terminator. strncmp stops at the first
   difference and at a null present in both, compares as unsigned characters,
   and never reads past the smaller bound, so a properly terminated value
   compares by its characters and an unterminated full array still gets an
   answer without reading past its end. */
int32_t oberon_str_cmp(const void *a, int32_t alen, const void *b, int32_t blen)
{
    size_t n = alen < blen ? (size_t)alen : (size_t)blen;
    return (int32_t)strncmp((const char *)a, (const char *)b, n);
}

/* Report 10.2 REAL operations. REAL is IEEE 754 binary32, so every one of
   these is the C float form of the same operation. */

float oberon_abs_real(float x) { return fabsf(x); }

/* FLOOR yields the largest INTEGER not greater than x, so a result must exist
   in the signed 32-bit range. The domain is checked here rather than left to
   the float-to-integer conversion, which no target has to define. Both
   endpoints are exact binary32 values. */
int32_t oberon_floor(float x, const char *module, int32_t line, int32_t col)
{
    if (!isfinite(x) || x < -2147483648.0f || x >= 2147483648.0f) {
        oberon_fail("FLOOR result is outside INTEGER range", module, line, col);
    }
    return (int32_t)floorf(x);
}

void oberon_pack(float *x, int32_t n) { *x = ldexpf(*x, (int)n); }

/* Report 10.2 normalizes the magnitude to [1, 2), while frexpf normalizes it
   to [0.5, 1), so the fraction doubles and the exponent drops by one. Zero
   satisfies no normalization interval; storing positive zero and a zero
   exponent round-trips through PACK and needs no invented exponent. */
void oberon_unpk(float *x, int32_t *n, const char *module, int32_t line, int32_t col)
{
    int exponent;

    if (!isfinite(*x)) {
        oberon_fail("UNPK argument is not finite", module, line, col);
    }
    if (*x == 0.0f) {
        *x = 0.0f;
        *n = 0;
        return;
    }
    *x = frexpf(*x, &exponent) * 2.0f;
    *n = (int32_t)(exponent - 1);
}

/* Traps. Generated code has already branched on the failing condition, so
   each one only reports where it came from. */

void oberon_div_by_zero(const char *module, int32_t line, int32_t col)
{
    oberon_fail("DIV or MOD by zero", module, line, col);
}

void oberon_assert_failed(const char *module, int32_t line, int32_t col)
{
    oberon_fail("assertion failed", module, line, col);
}

void oberon_abs_overflow(const char *module, int32_t line, int32_t col)
{
    oberon_fail("ABS overflows INTEGER", module, line, col);
}

void oberon_case_no_match(const char *module, int32_t line, int32_t col)
{
    oberon_fail("CASE without matching label", module, line, col);
}

void oberon_shift_range(const char *module, int32_t line, int32_t col)
{
    oberon_fail("shift count out of range", module, line, col);
}

void oberon_set_element_range(const char *module, int32_t line, int32_t col)
{
    oberon_fail("SET element out of range", module, line, col);
}

void oberon_byte_range(const char *module, int32_t line, int32_t col)
{
    oberon_fail("BYTE value out of range", module, line, col);
}

void oberon_chr_range(const char *module, int32_t line, int32_t col)
{
    oberon_fail("CHR argument is outside CHAR range", module, line, col);
}
