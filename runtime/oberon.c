#include <math.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <gc.h>

/* Generated code can hold an address that points into the middle of a heap
   object while nothing points at its first byte. Resolving the target of NEW is
   the ordinary case: `p.next := F()` computes the address of one field, then
   calls F, which may allocate and therefore may collect, and which may also
   have been what cleared p. Recognizing arbitrary interior pointers is what
   keeps the containing object alive across that call. The setting only takes
   effect before the collector initializes, so the two calls cannot be swapped.
   It is requested here rather than assumed, because whether it is on by default
   is a property of how the collector was built. */
void oberon_init(void)
{
    GC_set_all_interior_pointers(1);
    GC_INIT();
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

int32_t oberon_type_test_descriptor(
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

int32_t oberon_type_test_pointer(const void *pointer, const OberonTypeDescriptor *target)
{
    return pointer != NULL
        && oberon_type_test_descriptor(oberon_heap_descriptor(pointer), target);
}

/* Report 8.1: p^ and the implicit dereference in p.f both require p to point
   at a record, so a null pointer has no storage to select from. The check runs
   before the loaded value is used as an address. */
void oberon_check_nil(const void *pointer)
{
    if (pointer == NULL) {
        fputs("nil pointer dereference\n", stderr);
        exit(1);
    }
}

/* A procedure activation is not a data dereference, so it has its own stable
   failure instead of borrowing the nil-pointer diagnostic. Actual parameters
   have already been evaluated when generated code calls this check. */
void oberon_check_procedure(const void *procedure)
{
    if (procedure == NULL) {
        fputs("nil procedure call\n", stderr);
        exit(1);
    }
}

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

/* Report 9.1: an open array may be assigned to an array only when the whole
   source prefix fits. The check is separate from the move so failure cannot
   change even the first destination byte. */
void oberon_check_array_copy(int32_t source_length, int32_t destination_length)
{
    if (source_length > destination_length) {
        fputs("array assignment exceeds destination length\n", stderr);
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

/* Traps. None carries a source position yet; the runtime interface for source
   locations arrives in a later slice. */

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

void oberon_type_guard_failed(void)
{
    fputs("type guard failed\n", stderr);
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

void oberon_byte_range(void)
{
    fputs("BYTE value out of range\n", stderr);
    exit(1);
}

void oberon_chr_range(void)
{
    fputs("CHR argument is outside CHAR range\n", stderr);
    exit(1);
}
