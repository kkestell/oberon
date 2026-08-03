#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

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

/* Oakwood Out.Int(i, n): right-justified in a field of at least n spaces. */
void oberon_out_int(int32_t v, int32_t n) { printf("%*d", (int)n, (int)v); }
void oberon_out_ln(void)                  { putchar('\n'); }

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
