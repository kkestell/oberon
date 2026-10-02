#include <math.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <gc.h>

/* The command line is kept for the bundled Program module, which reads it
   through runtime/standard.c. */
int oberon_argc;
char **oberon_argv;

/* Generated code can hold an address that points into the middle of a heap
   object while nothing points at its first byte. Resolving the target of NEW is
   the ordinary case: `p.next := F()` computes the address of one field, then
   calls F, which may allocate and therefore may collect, and which may also
   have been what cleared p. Recognizing arbitrary interior pointers is what
   keeps the containing object alive across that call. The setting only takes
   effect before the collector initializes, so the two calls cannot be swapped.
   It is requested here rather than assumed, because whether it is on by default
   is a property of how the collector was built. */
void oberon_init(int argc, char **argv)
{
    oberon_argc = argc;
    oberon_argv = argv;
    GC_set_all_interior_pointers(1);
    GC_INIT();
}

/* Every language failure ends here. Out writes through the buffered C stdout,
   so it is flushed before the message goes to the unbuffered stderr: on a
   shared terminal the failure then follows the output the program wrote
   before it. The position is Module:line:column, the module standing in for
   the source path the executable does not know, with the 1-based line and
   column compile diagnostics use. exit flushes every other stream. */
static void oberon_fail(const char *message, const char *module, int64_t line, int64_t col)
{
    fflush(stdout);
    fprintf(stderr, "%s:%lld:%lld: %s\n", module, (long long)line, (long long)col, message);
    exit(1);
}

typedef struct OberonTypeDescriptor OberonTypeDescriptor;
struct OberonTypeDescriptor {
    const OberonTypeDescriptor *base;
};

static void *oberon_alloc_with_header(size_t n, const OberonTypeDescriptor *type, int atomic)
{
    size_t payload = n == 0 ? 1 : n;
    void *allocation = atomic
        ? GC_MALLOC_ATOMIC(sizeof(type) + payload)
        : GC_MALLOC(sizeof(type) + payload);
    if (allocation == NULL) {
        return NULL;
    }
    *(const OberonTypeDescriptor **)allocation = type;
    return (char *)allocation + sizeof(type);
}

void *oberon_alloc(size_t n, const OberonTypeDescriptor *type)
{
    return oberon_alloc_with_header(n, type, 0);
}

void *oberon_alloc_atomic(size_t n, const OberonTypeDescriptor *type)
{
    return oberon_alloc_with_header(n, type, 1);
}

const OberonTypeDescriptor *oberon_heap_descriptor(const void *pointer)
{
    return ((const OberonTypeDescriptor *const *)pointer)[-1];
}

int64_t oberon_type_test_descriptor(
    const OberonTypeDescriptor *actual,
    const OberonTypeDescriptor *target)
{
    while (actual != NULL) {
        if (actual == target) {
            return 1;
        }
        actual = actual->base;
    }
    return 0;
}

int64_t oberon_type_test_pointer(const void *pointer, const OberonTypeDescriptor *target)
{
    return pointer != NULL
        && oberon_type_test_descriptor(oberon_heap_descriptor(pointer), target);
}

/* Report 8.1: p^ and the implicit dereference in p.f both require p to point
   at a record, so a null pointer has no storage to select from. The check runs
   before the loaded value is used as an address. */
void oberon_check_nil(const void *pointer, const char *module, int64_t line, int64_t col)
{
    if (pointer == NULL) {
        oberon_fail("nil pointer dereference", module, line, col);
    }
}

/* A procedure activation is not a data dereference, so it has its own stable
   failure instead of borrowing the nil-pointer diagnostic. Actual parameters
   have already been evaluated when generated code calls this check. */
void oberon_check_procedure(
    const void *procedure,
    const char *module, int64_t line, int64_t col)
{
    if (procedure == NULL) {
        oberon_fail("nil procedure call", module, line, col);
    }
}

/* Report 8.1: an index must lie between zero and the length less one. The
   check runs before the element address is formed, so a zero-length array
   rejects every index and no invalid address is ever computed. */
int64_t oberon_check_index(
    int64_t index,
    int64_t length,
    const char *module, int64_t line, int64_t col)
{
    if (index < 0 || index >= length) {
        oberon_fail("array index out of bounds", module, line, col);
    }
    return index;
}

/* Report 9.1: an open array may be assigned to an array only when the whole
   source prefix fits. The check is separate from the move so failure cannot
   change even the first destination byte. */
void oberon_check_array_copy(
    int64_t source_length,
    int64_t destination_length,
    const char *module, int64_t line, int64_t col)
{
    if (source_length > destination_length) {
        oberon_fail("array assignment exceeds destination length", module, line, col);
    }
}

/* Report 9.1: an array assignment copies the value. memmove rather than memcpy
   because assigning a variable to itself is legal and has to mean something.
   A count of zero is the ordinary result of copying a zero-length array. */
void oberon_copy(void *destination, const void *source, size_t count)
{
    memmove(destination, source, count);
}

/* Report 8.2.4's relations on character arrays and strings. Each length
   bounds its operand: a character array's is its declared length and a
   string literal's counts its terminator. strncmp stops at the first
   difference and at a null present in both, compares as unsigned characters,
   and never reads past the smaller bound, so a properly terminated value
   compares by its characters and an unterminated full array still gets an
   answer without reading past its end. */
int64_t oberon_str_cmp(const void *a, int64_t alen, const void *b, int64_t blen)
{
    size_t n = alen < blen ? (size_t)alen : (size_t)blen;
    return (int64_t)strncmp((const char *)a, (const char *)b, n);
}

/* Report 10.2 REAL operations. REAL is IEEE 754 binary64, so every one of
   these is the C double form of the same operation. */

double oberon_abs_real(double x) { return fabs(x); }

/* FLOOR yields the largest INTEGER not greater than x, so a result must exist
   in the signed 64-bit range. The domain is checked here rather than left to
   the float-to-integer conversion, which no target has to define. Both
   endpoints are exact binary64 values. */
int64_t oberon_floor(double x, const char *module, int64_t line, int64_t col)
{
    if (!isfinite(x) || x < -9223372036854775808.0 || x >= 9223372036854775808.0) {
        oberon_fail("FLOOR result is outside INTEGER range", module, line, col);
    }
    return (int64_t)floor(x);
}

void oberon_pack(double *x, int64_t n) { *x = ldexp(*x, (int)n); }

/* Report 10.2 normalizes the magnitude to [1, 2), while frexp normalizes it
   to [0.5, 1), so the fraction doubles and the exponent drops by one. Zero
   satisfies no normalization interval; storing positive zero and a zero
   exponent round-trips through PACK and needs no invented exponent. */
void oberon_unpk(double *x, int64_t *n, const char *module, int64_t line, int64_t col)
{
    int exponent;

    if (!isfinite(*x)) {
        oberon_fail("UNPK argument is not finite", module, line, col);
    }
    if (*x == 0.0) {
        *x = 0.0;
        *n = 0;
        return;
    }
    *x = frexp(*x, &exponent) * 2.0;
    *n = (int64_t)(exponent - 1);
}

/* Traps. Generated code has already branched on the failing condition, so
   each one only reports where it came from. */

void oberon_div_by_zero(const char *module, int64_t line, int64_t col)
{
    oberon_fail("DIV or MOD by zero", module, line, col);
}

void oberon_assert_failed(const char *module, int64_t line, int64_t col)
{
    oberon_fail("assertion failed", module, line, col);
}

void oberon_abs_overflow(const char *module, int64_t line, int64_t col)
{
    oberon_fail("ABS overflows INTEGER", module, line, col);
}

void oberon_case_no_match(const char *module, int64_t line, int64_t col)
{
    oberon_fail("CASE without matching label", module, line, col);
}

void oberon_type_guard_failed(const char *module, int64_t line, int64_t col)
{
    oberon_fail("type guard failed", module, line, col);
}

void oberon_shift_range(const char *module, int64_t line, int64_t col)
{
    oberon_fail("shift count out of range", module, line, col);
}

void oberon_set_element_range(const char *module, int64_t line, int64_t col)
{
    oberon_fail("SET element out of range", module, line, col);
}

void oberon_byte_range(const char *module, int64_t line, int64_t col)
{
    oberon_fail("BYTE value out of range", module, line, col);
}

void oberon_chr_range(const char *module, int64_t line, int64_t col)
{
    oberon_fail("CHR argument is outside CHAR range", module, line, col);
}
