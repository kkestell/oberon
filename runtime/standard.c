#include <errno.h>
#include <math.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

void oberon_lib_out_char(int32_t ch) { putchar((unsigned char)ch); }

void oberon_lib_out_string(const unsigned char *s, int32_t length)
{
    int32_t i = 0;
    while (i < length && s[i] != 0) {
        putchar(s[i]);
        i++;
    }
}

void oberon_lib_out_int(int32_t value, int32_t width)
{
    printf("%*d", width > 0 ? width : 0, value);
}

void oberon_lib_out_hex(int32_t value) { printf(" %08X", (uint32_t)value); }

void oberon_lib_out_real(float value, int32_t width)
{
    printf("%*.6E", width > 0 ? width : 0, (double)value);
}

void oberon_lib_out_ln(void) { putchar('\n'); }

static int input_consumed;

static int ascii_space(int ch)
{
    return ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' || ch == '\f' || ch == '\v';
}

static int ascii_graph(int ch) { return (ch >= 33 && ch <= 126) || ch >= 128; }

static int read_byte(void)
{
    int ch = getchar();
    if (ch != EOF) {
        input_consumed = 1;
    }
    return ch;
}

static int first_token_byte(void)
{
    int ch;
    do {
        ch = read_byte();
    } while (ch != EOF && ascii_space(ch));
    return ch;
}

static size_t read_word(unsigned char *buffer, size_t capacity)
{
    int ch = first_token_byte();
    size_t length = 0;
    while (ch != EOF && !ascii_space(ch)) {
        if (length < capacity) {
            buffer[length] = (unsigned char)ch;
        }
        length++;
        ch = read_byte();
    }
    if (ch != EOF) {
        ungetc(ch, stdin);
    }
    return length;
}

int32_t oberon_lib_in_open(void) { return !input_consumed; }

int32_t oberon_lib_in_char(unsigned char *ch)
{
    int value = read_byte();
    if (value == EOF) {
        return 0;
    }
    *ch = (unsigned char)value;
    return 1;
}

int32_t oberon_lib_in_int(int32_t *result)
{
    unsigned char token[128];
    size_t length = read_word(token, sizeof token);
    size_t i = 0;
    int negative = 0;
    uint64_t value = 0;

    if (length == 0 || length > sizeof token) {
        return 0;
    }
    if (token[i] == '-') {
        negative = 1;
        i++;
    }
    if (i == length) {
        return 0;
    }

    if (!negative && token[length - 1] == 'H') {
        size_t digits = length - 1;
        if (digits == 0 || digits > 8) {
            return 0;
        }
        for (i = 0; i < digits; i++) {
            unsigned char ch = token[i];
            unsigned digit;
            if (ch >= '0' && ch <= '9') {
                digit = ch - '0';
            } else if (ch >= 'A' && ch <= 'F') {
                digit = ch - 'A' + 10;
            } else {
                return 0;
            }
            value = value * 16 + digit;
        }
        *result = (int32_t)(uint32_t)value;
        return 1;
    }

    for (; i < length; i++) {
        if (token[i] < '0' || token[i] > '9') {
            return 0;
        }
        value = value * 10 + (token[i] - '0');
        if ((!negative && value > INT32_MAX) || (negative && value > UINT64_C(2147483648))) {
            return 0;
        }
    }
    *result = negative ? (value == UINT64_C(2147483648) ? INT32_MIN : -(int32_t)value)
                       : (int32_t)value;
    return 1;
}

int32_t oberon_lib_in_real(float *result)
{
    unsigned char token[256];
    size_t length = read_word(token, sizeof token - 1);
    size_t i = 0;
    char *end;
    float value;

    if (length == 0 || length >= sizeof token) {
        return 0;
    }
    if (token[i] == '+' || token[i] == '-') {
        i++;
    }
    if (i == length || token[i] < '0' || token[i] > '9') {
        return 0;
    }
    while (i < length && token[i] >= '0' && token[i] <= '9') {
        i++;
    }
    if (i == length || token[i] != '.') {
        return 0;
    }
    i++;
    while (i < length && token[i] >= '0' && token[i] <= '9') {
        i++;
    }
    if (i < length && token[i] == 'E') {
        i++;
        if (i < length && (token[i] == '+' || token[i] == '-')) {
            i++;
        }
        if (i == length || token[i] < '0' || token[i] > '9') {
            return 0;
        }
        while (i < length && token[i] >= '0' && token[i] <= '9') {
            i++;
        }
    }
    if (i != length) {
        return 0;
    }
    token[length] = 0;
    value = strtof((const char *)token, &end);
    if (end != (char *)token + length || !isfinite(value)) {
        return 0;
    }
    *result = value;
    return 1;
}

int32_t oberon_lib_in_string(unsigned char *destination, int32_t capacity)
{
    int ch = first_token_byte();
    int32_t stored = 0;
    int32_t count = 0;
    int valid = 1;

    if (capacity <= 0) {
        if (ch == '"') {
            do {
                ch = read_byte();
            } while (ch != EOF && ch != '"' && ch != '\n');
            if (ch == '"') {
                ch = read_byte();
                while (ch != EOF && !ascii_space(ch)) {
                    ch = read_byte();
                }
                if (ch != EOF) {
                    ungetc(ch, stdin);
                }
            }
        } else {
            while (ch != EOF && !ascii_space(ch)) {
                ch = read_byte();
            }
            if (ch != EOF) {
                ungetc(ch, stdin);
            }
        }
        return 0;
    }

    if (ch == '"') {
        ch = read_byte();
        while (ch != EOF && ch != '"' && ch != '\n') {
            if (stored < capacity - 1) {
                destination[stored] = (unsigned char)ch;
                stored++;
            }
            count++;
            ch = read_byte();
        }
        valid = ch == '"';
        if (valid) {
            ch = read_byte();
            if (ch != EOF && ascii_space(ch)) {
                ungetc(ch, stdin);
            } else if (ch != EOF) {
                valid = 0;
                while (ch != EOF && !ascii_space(ch)) {
                    ch = read_byte();
                }
                if (ch != EOF) {
                    ungetc(ch, stdin);
                }
            }
        }
        valid = valid && count < capacity;
    } else {
        unsigned char token[128];
        size_t length = 0;
        uint32_t ordinal = 0;
        while (ch != EOF && !ascii_space(ch)) {
            if (length < sizeof token) {
                token[length] = (unsigned char)ch;
            }
            length++;
            ch = read_byte();
        }
        if (ch != EOF) {
            ungetc(ch, stdin);
        }
        valid = length >= 2 && length <= sizeof token && token[length - 1] == 'X';
        for (size_t i = 0; valid && i + 1 < length; i++) {
            unsigned char digit = token[i];
            if (digit >= '0' && digit <= '9') {
                ordinal = ordinal * 16 + digit - '0';
            } else if (digit >= 'A' && digit <= 'F') {
                ordinal = ordinal * 16 + digit - 'A' + 10;
            } else {
                valid = 0;
            }
            if (ordinal > 255) {
                valid = 0;
            }
        }
        valid = valid && capacity >= 2;
        if (valid) {
            destination[0] = (unsigned char)ordinal;
            destination[1] = 0;
            return 1;
        }
    }

    if (valid) {
        destination[stored] = 0;
        return 1;
    }
    destination[0] = 0;
    return 0;
}

int32_t oberon_lib_in_name(unsigned char *destination, int32_t capacity)
{
    int ch = first_token_byte();
    int32_t count = 0;
    int valid = ch != EOF;
    while (ch != EOF && !ascii_space(ch)) {
        if (!ascii_graph(ch)) {
            valid = 0;
        }
        if (capacity > 0 && count < capacity - 1) {
            destination[count] = (unsigned char)ch;
        }
        count++;
        ch = read_byte();
    }
    if (capacity <= 0) {
        return 0;
    }
    if (valid && count < capacity) {
        destination[count] = 0;
        return 1;
    }
    destination[0] = 0;
    return 0;
}

int32_t oberon_lib_in_line(unsigned char *destination, int32_t capacity)
{
    int ch = read_byte();
    int32_t count = 0;
    int available = 0;
    while (ch != EOF && ch != '\n') {
        available = 1;
        if (capacity > 0 && count < capacity - 1) {
            destination[count] = (unsigned char)ch;
        }
        count++;
        ch = read_byte();
    }
    if (ch == '\n') {
        available = 1;
    }
    if (capacity <= 0) {
        return 0;
    }
    destination[count < capacity ? count : capacity - 1] = 0;
    return available && count < capacity;
}

float oberon_lib_math_sqrt(float x) { return sqrtf(x); }
float oberon_lib_math_power(float x, float y) { return powf(x, y); }
float oberon_lib_math_exp(float x) { return expf(x); }
float oberon_lib_math_ln(float x) { return logf(x); }
float oberon_lib_math_log(float x, float base) { return logf(x) / logf(base); }
float oberon_lib_math_round(float x) { return floorf(x + 0.5f); }
float oberon_lib_math_sin(float x) { return sinf(x); }
float oberon_lib_math_cos(float x) { return cosf(x); }
float oberon_lib_math_tan(float x) { return tanf(x); }
float oberon_lib_math_arcsin(float x) { return asinf(x); }
float oberon_lib_math_arccos(float x) { return acosf(x); }
float oberon_lib_math_arctan(float x) { return atanf(x); }
float oberon_lib_math_arctan2(float y, float x) { return atan2f(y, x); }
float oberon_lib_math_sinh(float x) { return sinhf(x); }
float oberon_lib_math_cosh(float x) { return coshf(x); }
float oberon_lib_math_tanh(float x) { return tanhf(x); }
float oberon_lib_math_arcsinh(float x) { return asinhf(x); }
float oberon_lib_math_arccosh(float x) { return acoshf(x); }
float oberon_lib_math_arctanh(float x) { return atanhf(x); }

#define FILE_LIMIT 1024
#define ERROR_LIMIT 256
#define STATUS_OK 0
#define STATUS_EOF 1
#define STATUS_ERROR 2
#define STATUS_INVALID 3

typedef struct {
    FILE *stream;
    char *name;
    int registered;
} LibraryFile;

static LibraryFile files[FILE_LIMIT];
static char last_error[ERROR_LIMIT];

static void set_error(const char *format, ...)
{
    va_list args;
    va_start(args, format);
    vsnprintf(last_error, sizeof last_error, format, args);
    va_end(args);
}

static char *bounded_string(const unsigned char *source, int32_t length)
{
    const unsigned char *end;
    char *copy;
    size_t count;
    if (length < 0 || (end = memchr(source, 0, (size_t)length)) == NULL) {
        set_error("unterminated file name");
        return NULL;
    }
    count = (size_t)(end - source);
    copy = malloc(count + 1);
    if (copy == NULL) {
        set_error("out of memory");
        return NULL;
    }
    memcpy(copy, source, count);
    copy[count] = 0;
    return copy;
}

static int32_t store_file(FILE *stream, char *name, int registered)
{
    for (int32_t id = 1; id < FILE_LIMIT; id++) {
        if (files[id].stream == NULL) {
            files[id].stream = stream;
            files[id].name = name;
            files[id].registered = registered;
            return id;
        }
    }
    set_error("too many open files");
    fclose(stream);
    free(name);
    return 0;
}

static LibraryFile *get_file(int32_t id)
{
    if (id <= 0 || id >= FILE_LIMIT || files[id].stream == NULL) {
        set_error("invalid file handle");
        return NULL;
    }
    return &files[id];
}

int32_t oberon_lib_file_old(const unsigned char *source, int32_t length)
{
    char *name = bounded_string(source, length);
    struct stat information;
    FILE *stream;
    if (name == NULL) {
        return 0;
    }
    if (stat(name, &information) != 0 || !S_ISREG(information.st_mode)) {
        set_error("cannot open '%s': %s", name, errno ? strerror(errno) : "not a regular file");
        free(name);
        return 0;
    }
    stream = fopen(name, "r+b");
    if (stream == NULL) {
        stream = fopen(name, "rb");
    }
    if (stream == NULL) {
        set_error("cannot open '%s': %s", name, strerror(errno));
        free(name);
        return 0;
    }
    return store_file(stream, name, 1);
}

int32_t oberon_lib_file_new(const unsigned char *source, int32_t length)
{
    char *name = bounded_string(source, length);
    FILE *stream;
    if (name == NULL) {
        return 0;
    }
    stream = tmpfile();
    if (stream == NULL) {
        set_error("cannot create temporary file: %s", strerror(errno));
        free(name);
        return 0;
    }
    return store_file(stream, name, 0);
}

void oberon_lib_file_release(int32_t id)
{
    LibraryFile *file = get_file(id);
    if (file != NULL) {
        fclose(file->stream);
        free(file->name);
        memset(file, 0, sizeof *file);
    }
}

void oberon_lib_file_register(int32_t id)
{
    LibraryFile *file = get_file(id);
    FILE *destination;
    unsigned char buffer[4096];
    int failed = 0;
    if (file == NULL || file->registered) {
        return;
    }
    destination = fopen(file->name, "w+b");
    if (destination == NULL) {
        set_error("cannot register '%s': %s", file->name, strerror(errno));
        return;
    }
    if (fseek(file->stream, 0, SEEK_SET) != 0) {
        failed = 1;
    }
    while (!failed) {
        size_t count = fread(buffer, 1, sizeof buffer, file->stream);
        if (count != 0 && fwrite(buffer, 1, count, destination) != count) {
            failed = 1;
        }
        if (count < sizeof buffer) {
            if (ferror(file->stream)) {
                failed = 1;
            }
            break;
        }
    }
    if (!failed && fflush(destination) != 0) {
        failed = 1;
    }
    if (failed) {
        set_error("cannot register '%s': %s", file->name, strerror(errno));
        fclose(destination);
        clearerr(file->stream);
        return;
    }
    fclose(file->stream);
    file->stream = destination;
    file->registered = 1;
}

void oberon_lib_file_close(int32_t id)
{
    LibraryFile *file = get_file(id);
    if (file != NULL && fflush(file->stream) != 0) {
        set_error("cannot flush '%s': %s", file->name, strerror(errno));
        clearerr(file->stream);
    }
}

void oberon_lib_file_purge(int32_t id)
{
    LibraryFile *file = get_file(id);
    if (file != NULL && (fflush(file->stream) != 0 || ftruncate(fileno(file->stream), 0) != 0)) {
        set_error("cannot purge '%s': %s", file->name, strerror(errno));
        clearerr(file->stream);
    }
}

int32_t oberon_lib_file_delete(const unsigned char *source, int32_t length)
{
    char *name = bounded_string(source, length);
    int result;
    if (name == NULL) {
        return 1;
    }
    result = unlink(name);
    if (result != 0) {
        set_error("cannot delete '%s': %s", name, strerror(errno));
    }
    free(name);
    return result;
}

int32_t oberon_lib_file_rename(
    const unsigned char *old_source,
    int32_t old_length,
    const unsigned char *new_source,
    int32_t new_length)
{
    char *old_name = bounded_string(old_source, old_length);
    char *new_name = bounded_string(new_source, new_length);
    int result;
    if (old_name == NULL || new_name == NULL) {
        free(old_name);
        free(new_name);
        return 1;
    }
    result = rename(old_name, new_name);
    if (result != 0) {
        set_error("cannot rename '%s': %s", old_name, strerror(errno));
    } else {
        for (int id = 1; id < FILE_LIMIT; id++) {
            if (files[id].stream != NULL && files[id].registered && strcmp(files[id].name, old_name) == 0) {
                char *copy = strdup(new_name);
                if (copy != NULL) {
                    free(files[id].name);
                    files[id].name = copy;
                }
            }
        }
    }
    free(old_name);
    free(new_name);
    return result;
}

int32_t oberon_lib_file_length(int32_t id)
{
    LibraryFile *file = get_file(id);
    long length;
    if (file == NULL || fseek(file->stream, 0, SEEK_END) != 0 || (length = ftell(file->stream)) < 0
        || length > INT32_MAX) {
        if (file != NULL) {
            set_error("cannot get length of '%s': %s", file->name, strerror(errno));
            clearerr(file->stream);
        }
        return 0;
    }
    return (int32_t)length;
}

int32_t oberon_lib_file_date(int32_t id, int32_t *time_value, int32_t *date_value)
{
    LibraryFile *file = get_file(id);
    struct stat information;
    struct tm local;
    int32_t new_time;
    int32_t new_date;
    if (file == NULL || fstat(fileno(file->stream), &information) != 0
        || localtime_r(&information.st_mtime, &local) == NULL) {
        if (file != NULL) {
            set_error("cannot get date of '%s': %s", file->name, strerror(errno));
        }
        return 0;
    }
    new_time = local.tm_hour * 4096 + local.tm_min * 64 + local.tm_sec;
    new_date = (local.tm_year + 1900) * 512 + (local.tm_mon + 1) * 32 + local.tm_mday;
    *time_value = new_time;
    *date_value = new_date;
    return 1;
}

static int position(LibraryFile *file, int32_t offset)
{
    clearerr(file->stream);
    if (offset < 0 || fseek(file->stream, offset, SEEK_SET) != 0) {
        set_error("cannot position '%s': %s", file->name, strerror(errno));
        clearerr(file->stream);
        return 0;
    }
    return 1;
}

static int32_t read_data(int32_t id, int32_t offset, void *destination, int32_t count, int32_t *moved)
{
    LibraryFile *file = get_file(id);
    unsigned char staging[16];
    size_t actual;
    *moved = 0;
    if (file == NULL || count < 0 || count > (int32_t)sizeof staging || !position(file, offset)) {
        return STATUS_ERROR;
    }
    actual = fread(staging, 1, (size_t)count, file->stream);
    *moved = (int32_t)actual;
    if (actual == (size_t)count) {
        memcpy(destination, staging, (size_t)count);
        return STATUS_OK;
    }
    if (ferror(file->stream)) {
        set_error("cannot read '%s': %s", file->name, strerror(errno));
        clearerr(file->stream);
        return STATUS_ERROR;
    }
    set_error("unexpected end of file '%s'", file->name);
    return STATUS_EOF;
}

static int32_t write_data(int32_t id, int32_t offset, const void *source, int32_t count, int32_t *moved)
{
    LibraryFile *file = get_file(id);
    size_t actual;
    *moved = 0;
    if (file == NULL || count < 0 || !position(file, offset)) {
        return STATUS_ERROR;
    }
    actual = fwrite(source, 1, (size_t)count, file->stream);
    *moved = (int32_t)actual;
    if (actual == (size_t)count) {
        return STATUS_OK;
    }
    set_error("cannot write '%s': %s", file->name, strerror(errno));
    clearerr(file->stream);
    return STATUS_ERROR;
}

int32_t oberon_lib_file_read_byte(int32_t id, int32_t pos, unsigned char *x, int32_t *moved)
{
    return read_data(id, pos, x, 1, moved);
}
int32_t oberon_lib_file_read_int(int32_t id, int32_t pos, int32_t *x, int32_t *moved)
{
    return read_data(id, pos, x, 4, moved);
}
int32_t oberon_lib_file_read_real(int32_t id, int32_t pos, float *x, int32_t *moved)
{
    return read_data(id, pos, x, 4, moved);
}
int32_t oberon_lib_file_read_set(int32_t id, int32_t pos, uint32_t *x, int32_t *moved)
{
    return read_data(id, pos, x, 4, moved);
}

int32_t oberon_lib_file_read_bool(int32_t id, int32_t pos, int32_t *x, int32_t *moved)
{
    unsigned char byte;
    int32_t status = read_data(id, pos, &byte, 1, moved);
    if (status != STATUS_OK) {
        return status;
    }
    if (byte > 1) {
        set_error("invalid BOOLEAN encoding");
        return STATUS_INVALID;
    }
    *x = byte;
    return STATUS_OK;
}

int32_t oberon_lib_file_read_num(int32_t id, int32_t pos, int32_t *x, int32_t *moved)
{
    LibraryFile *file = get_file(id);
    uint64_t low = 0;
    int shift = 0;
    *moved = 0;
    if (file == NULL || !position(file, pos)) {
        return STATUS_ERROR;
    }
    for (;;) {
        int ch = fgetc(file->stream);
        if (ch == EOF) {
            if (ferror(file->stream)) {
                set_error("cannot read '%s': %s", file->name, strerror(errno));
                clearerr(file->stream);
                return STATUS_ERROR;
            }
            set_error("unterminated compact INTEGER encoding");
            return STATUS_EOF;
        }
        (*moved)++;
        if (ch >= 128) {
            if (shift >= 35 || (shift == 28 && (ch - 128) > 15)) {
                while (ch >= 128) {
                    ch = fgetc(file->stream);
                    if (ch == EOF) {
                        if (ferror(file->stream)) {
                            set_error("cannot read '%s': %s", file->name, strerror(errno));
                            clearerr(file->stream);
                            return STATUS_ERROR;
                        }
                        set_error("unterminated compact INTEGER encoding");
                        return STATUS_EOF;
                    }
                    (*moved)++;
                }
                set_error("invalid compact INTEGER encoding");
                return STATUS_INVALID;
            }
            low |= (uint64_t)(ch - 128) << shift;
            shift += 7;
        } else {
            int64_t high = ch & 63;
            if (ch >= 64) {
                high -= 64;
            }
            int64_t value = (int64_t)low + high * (INT64_C(1) << shift);
            if (value < INT32_MIN || value > INT32_MAX || shift > 28) {
                set_error("invalid compact INTEGER encoding");
                return STATUS_INVALID;
            }
            *x = (int32_t)value;
            return STATUS_OK;
        }
    }
}

int32_t oberon_lib_file_read_string(
    int32_t id,
    int32_t pos,
    unsigned char *destination,
    int32_t capacity,
    int32_t *moved)
{
    LibraryFile *file = get_file(id);
    int32_t stored = 0;
    int fits = 1;
    *moved = 0;
    if (file == NULL || capacity <= 0 || !position(file, pos)) {
        return STATUS_ERROR;
    }
    for (;;) {
        int ch = fgetc(file->stream);
        if (ch == EOF) {
            destination[0] = 0;
            if (ferror(file->stream)) {
                set_error("cannot read '%s': %s", file->name, strerror(errno));
                clearerr(file->stream);
                return STATUS_ERROR;
            }
            set_error("unterminated stored string");
            return STATUS_EOF;
        }
        (*moved)++;
        if (ch == 0) {
            if (fits) {
                destination[stored] = 0;
                return STATUS_OK;
            }
            destination[0] = 0;
            set_error("stored string does not fit destination");
            return STATUS_INVALID;
        }
        if (stored < capacity - 1) {
            destination[stored++] = (unsigned char)ch;
        } else {
            fits = 0;
        }
    }
}

int32_t oberon_lib_file_read_bytes(
    int32_t id,
    int32_t pos,
    unsigned char *destination,
    int32_t capacity,
    int32_t requested,
    int32_t *moved)
{
    LibraryFile *file = get_file(id);
    int32_t count = requested < capacity ? requested : capacity;
    size_t actual;
    *moved = 0;
    if (file == NULL || requested < 0 || capacity < 0 || !position(file, pos)) {
        return STATUS_ERROR;
    }
    actual = fread(destination, 1, (size_t)count, file->stream);
    *moved = (int32_t)actual;
    if (actual == (size_t)count) {
        return STATUS_OK;
    }
    if (ferror(file->stream)) {
        set_error("cannot read '%s': %s", file->name, strerror(errno));
        clearerr(file->stream);
        return STATUS_ERROR;
    }
    set_error("unexpected end of file '%s'", file->name);
    return STATUS_EOF;
}

int32_t oberon_lib_file_write_byte(int32_t id, int32_t pos, int32_t x, int32_t *moved)
{
    unsigned char byte = (unsigned char)x;
    return write_data(id, pos, &byte, 1, moved);
}
int32_t oberon_lib_file_write_int(int32_t id, int32_t pos, int32_t x, int32_t *moved)
{
    return write_data(id, pos, &x, 4, moved);
}
int32_t oberon_lib_file_write_real(int32_t id, int32_t pos, float x, int32_t *moved)
{
    return write_data(id, pos, &x, 4, moved);
}
int32_t oberon_lib_file_write_set(int32_t id, int32_t pos, uint32_t x, int32_t *moved)
{
    return write_data(id, pos, &x, 4, moved);
}
int32_t oberon_lib_file_write_bool(int32_t id, int32_t pos, int32_t x, int32_t *moved)
{
    unsigned char byte = x ? 1 : 0;
    return write_data(id, pos, &byte, 1, moved);
}

int32_t oberon_lib_file_write_num(int32_t id, int32_t pos, int32_t x, int32_t *moved)
{
    unsigned char bytes[5];
    int count = 0;
    int32_t value = x;
    while (value < -64 || value > 63) {
        int32_t remainder = value % 128;
        if (remainder < 0) {
            remainder += 128;
        }
        bytes[count++] = (unsigned char)(remainder + 128);
        value = (value - remainder) / 128;
    }
    int32_t remainder = value % 128;
    if (remainder < 0) {
        remainder += 128;
    }
    bytes[count++] = (unsigned char)remainder;
    return write_data(id, pos, bytes, count, moved);
}

int32_t oberon_lib_file_write_string(
    int32_t id,
    int32_t pos,
    const unsigned char *source,
    int32_t length,
    int32_t *moved)
{
    const unsigned char *end = length < 0 ? NULL : memchr(source, 0, (size_t)length);
    if (end == NULL) {
        *moved = 0;
        set_error("unterminated source string");
        return STATUS_INVALID;
    }
    return write_data(id, pos, source, (int32_t)(end - source) + 1, moved);
}

int32_t oberon_lib_file_write_bytes(
    int32_t id,
    int32_t pos,
    unsigned char *source,
    int32_t capacity,
    int32_t requested,
    int32_t *moved)
{
    if (requested < 0 || requested > capacity) {
        *moved = 0;
        set_error("invalid byte transfer count");
        return STATUS_INVALID;
    }
    return write_data(id, pos, source, requested, moved);
}

void oberon_lib_file_error(unsigned char *destination, int32_t capacity)
{
    if (capacity > 0) {
        size_t count = strlen(last_error);
        if (count >= (size_t)capacity) {
            count = (size_t)capacity - 1;
        }
        memcpy(destination, last_error, count);
        destination[count] = 0;
    }
}
