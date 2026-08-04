# Architecture

```text
Root source file
    ↓
Module discovery  ──→  for each module: Lexer → Parser → AST → Semantic Analysis
    ↓
Aggregate typed IR
    ↓
QBE IL
    ↓
QBE
    ↓
System Assembler + Linker
    ↓
Native Executable
```

The compiler uses a small typed intermediate representation between semantic analysis and code generation. The IR makes addresses, values, storage, calls, and runtime operations explicit while remaining much smaller than LLVM IR.

## Modules

A build starts from one root source file and compiles every module that file reaches through imports. Imports are followed depth first, in the order they are written, and a module reached twice is compiled once. An import naming a module whose compilation has already started is an import cycle and ends the build.

Each module is compiled completely before any module that imports it. Analysis therefore returns two things: the module's IR and its interface. The interface holds only the declarations carrying an export mark, so a client importing the module can see nothing else.

### Source lookup

The root source file's directory is the build's root directory. An import of module `M` looks for a file named exactly `M.Mod` in two places, in this order:

1. The root directory.
2. The bundled `lib/` directory.

The root directory comes first so an application can supply a module that shadows a bundled one. If neither directory has the file, the module name `Out` falls back to a temporary interface backed by the C runtime. That fallback is the last piece of the standard library with no Oberon source, and it goes away when `lib/Out.Mod` arrives.

Every source module must live in a file named after it. There are no search paths, packages, or serialized interfaces, and nothing is cached between invocations: each build rereads and reanalyzes every source it needs.

### Initialization

All source modules of one build are emitted into a single QBE unit, so a call from one to another needs no linkage annotation. Export marks are a semantic rule that analysis has already applied by the time code is generated.

The generated `main` calls `oberon_init` and then calls each module's initializer once, in the order the modules were compiled. Because a dependency finishes before its client starts, that order runs every module body after the bodies of everything it imports, and the root module's body last.

## Scalar representation

`INTEGER` is a signed 32-bit number, `BOOLEAN` is 0 or 1, and `SET` is a 32-bit bit vector; all three occupy four bytes and travel in a QBE word. `REAL` is an IEEE 754 binary32 value and travels in a QBE single, which is the platform's native C `float` calling class. Not every scalar is four bytes: `CHAR` and `BYTE` occupy one unsigned byte each, load zero-extended with `loadub`, store with `storeb`, and still travel in a QBE word in every calling position. All six remain distinct semantic types: sharing a machine class never makes one assignable to another, and `INTEGER` and `REAL` never mix implicitly even though both are numeric.

A pointer occupies eight bytes, has alignment eight, and travels in QBE's long class. Pointer storage loads with `loadl` and stores with `storel`. A pointer value is distinct from the address of the variable that stores it, so a value parameter receives and copies the pointer while a `VAR` parameter receives the address of pointer storage. Pointer function results also use the long class. A procedure value uses the same pointer-sized machine class but remains a distinct IR and semantic type.

### Pointers and `NIL`

Each `POINTER` constructor creates one semantic type identity. Aliases and imported interfaces share that identity. Pointer types inherit the extension relation of their bound records, including across separately written pointer constructors. Assignment and value parameters accept a pointer whose base record extends the destination's base record. Equality and inequality accept two pointer types when either one extends the other. A pointer `VAR` formal keeps the stricter identical-type rule.

A pointer base must be a record. While a `TYPE` section is being analyzed, an otherwise undeclared bare name used as a pointer base becomes a pending same-scope reference. Pending bases are resolved after that section. Ordinary lexical lookup runs first, qualified names never become pending, and every missing or non-record base is diagnosed before variables or statements are lowered.

Recursive source types form bounded shared descriptor graphs. A pointer descriptor holds a mutable pending, resolved, or invalid base state, and a record descriptor owns its field types. Debug formatting stays shallow so a recursive record and pointer pair cannot recurse while being printed.

`NIL` is a polymorphic constant whose machine value is zero. It has a semantic pseudo-type but no source type name or storage layout. It is compatible with every pointer and procedure type for assignment, value arguments, results, equality, and inequality. It is not an integer zero and is rejected in arithmetic, ordering, conditions, indexing, `CASE`, and other unrelated scalar operations.

Explicit `p^` and implicit field selection through `p.f` use one checked dereference operation. The compiler loads the pointer once, emits `oberon_check_nil`, and only then treats the checked value as the address of its base record. The checked pointer also makes the heap object's dynamic record descriptor recoverable without loading it until a polymorphic call or dynamic-type operation needs it. Read-only status follows the address through dereference, fields, and indices. Short-circuit Boolean evaluation remains the way a source program guards a dereference.

Fixed `LEN` has separate executable and required-constant paths. Executable `LEN(p.row)` resolves the designator and therefore executes nil and index checks before returning the declared length. Required-constant `LEN` follows selector types without loading pointers or evaluating indices; it still type-checks every index and diagnoses a constant index outside its fixed bound.

### Procedure types and indirect calls

Each `PROCEDURE` type constructor has one identity and owns an ordered source signature. A formal records its value or `VAR` mode and resolved type, while the signature records an optional result. Formal names are checked for duplicates and then discarded. Procedure declarations use this same representation, paired with their direct code symbol, so declarations, variables, parameters, results, and exported interfaces cannot develop separate notions of one callable shape.

Assignment, value substitution, equality, and inequality compare procedure signatures structurally. Proper and function procedures must agree, and formal counts, order, modes, and types must match. Open-array dimensions are compared recursively, as are procedure-typed formals. The component types inside a signature retain their existing identities, so separately constructed fixed arrays, records, and pointers do not become compatible merely because their layouts match. A function result component also requires constructor identity, including when that result is itself a procedure type. A procedure `VAR` formal applies the ordinary identical-type rule to its actual, so two separately constructed but structurally matching procedure types are not interchangeable in that context.

A procedure value is one eight-byte code address, or zero for `NIL`. It loads and stores with QBE's long operations and can occupy globals, locals, parameters, results, array elements, and record fields. It is not a data pointer, does not make an aggregate pointer-containing, and is never a GC root by virtue of its source type. No descriptor, registry, trampoline, static link, or closure object exists.

Only a procedure declared in a module scope can become a value. The rule includes exported procedures reached through an imported module and the runtime-backed `Out` members because those members are ordinary module procedures. Nested and predefined procedures remain directly callable but cannot be assigned, passed, returned, or compared as values. This restriction eliminates the need for closures while retaining direct recursion and nested direct calls.

A procedure value parameter receives one copied code address and lives in a writable local slot like another scalar value parameter. A procedure `VAR` parameter receives the address of writable procedure storage. A function may return a basic, pointer, or procedure value; arrays, open arrays, and records remain invalid results. An imported procedure variable can be read, compared, passed by value, and called, but its read-only status prevents assignment and `VAR` substitution.

The typed IR represents a call target as either a direct symbol or an indirect value. Both forms use the same argument builder and result representation. Structured values therefore remain addresses, every open dimension contributes its length in the same position, and every record `VAR` address is followed immediately by its dynamic descriptor regardless of how the callee was reached.

An indirect call resolves all selectors and loads its procedure value exactly once before evaluating actual parameters. Actuals then evaluate from left to right. The call uses the captured value even when an actual changes the original procedure variable. After all actuals have run, `oberon_check_procedure` rejects a zero target by printing `nil procedure call` and exiting; a nonzero target is called through its captured QBE temporary. Direct calls need no dynamic check and keep their direct-symbol form.

Semantic analysis decides whether a terminal parenthesized postfix is a call or a type guard from the prefix type. A procedure-valued place makes it an indirect call. An eligible record or pointer place makes it a guard. Field, index, dereference, and guard selectors can all precede the terminal call, but the grammar permits no selector after an actual-parameter list.

### REAL

Project Oberon gives `REAL` four bytes and operates on the binary32 sign, exponent, and fraction fields. QBE defines its `s` type as an IEEE 754 32-bit float and passes it in the platform's floating-point class. The two agree, so no compiler-specific calling convention is needed and no target description or configurable floating-point layer exists.

A decimal literal is parsed straight to binary32 in the lexer, so the value the AST carries is the value the generated code uses and nothing rounds twice. A literal whose magnitude has no finite binary32 representation is a source diagnostic; one too small to represent rounds to zero, which is the ordinary IEEE result.

Arithmetic rounds at binary32 at every source operator, and constant folding uses the same `f32` operations at the same points, so a folded expression and the same expression computed at run time agree. `REAL` arithmetic is plain IEEE 754: overflow yields an infinity, division by zero yields an infinity or a NaN, and neither traps, because the Report defines no trap for them. A NaN is unequal to every value including itself, and every ordering comparison involving one is false. Source has no spelling for an infinity or a NaN; both only arise from arithmetic.

A `REAL` immediate reaches QBE as the signed decimal spelling of its 32-bit pattern. A QBE constant is an untyped bit string, so in an `s` context that reproduces every binary32 value exactly, including negative zero and the non-finite values constant folding can produce.

A `REAL` module variable is a four-byte zero-filled data object, so its initial value is positive zero. A `REAL` local, like every other local, is uninitialized until assigned. A `REAL` value parameter is a writable local copy, and a `REAL` variable parameter is an address and uses the QBE pointer class.

### REAL operations

`ABS` accepts an `INTEGER` or a `REAL` and returns the argument's type. The `INTEGER` form keeps its overflow trap at `MIN(INTEGER)`; the `REAL` form calls `oberon_abs_real`, which clears the sign of a negative zero, leaves an infinity alone, and returns a NaN for a NaN.

`FLT` is the explicit `INTEGER`-to-`REAL` conversion and is the only conversion between machine classes the compiler has, so it emits QBE `swtof` and needs no general conversion matrix. It rounds to the nearest representable value and does not promise that every `INTEGER` survives: `FLT(MAX(INTEGER))` is `2147483648.0`. No assignment or arithmetic context inserts it; every conversion is written in the source.

`FLOOR` yields the largest `INTEGER` not greater than its argument, rounding toward negative infinity, so `FLOOR(-1.5)` is -2. A result exists only when the rounded argument is finite and lies from `-2147483648.0` inclusive to `2147483648.0` exclusive. A constant outside that domain is a source diagnostic. A dynamic one calls `oberon_floor`, which checks finiteness and both endpoints itself rather than relying on a float-to-integer conversion no target has to define, and which prints `FLOOR result is outside INTEGER range` and exits. The domain applies after binary32 rounding, so `FLOOR(FLT(MAX(INTEGER)))` is out of range.

`PACK(x, n)` replaces `x` with `x * 2^n` through `oberon_pack`, a wrapper around `ldexpf`. Overflow, subnormal results, and underflow to zero are ordinary binary32 outcomes and do not trap.

`UNPK(x, n)` is its inverse: it preserves the sign of `x`, normalizes the magnitude into 1 through 2 excluding 2, and stores the base-two exponent in `n`. The Report states the interval without resolving a negative argument, so this compiler follows OBNC and applies the interval to the absolute value. Project Oberon instead folds the sign into an encoded exponent, which round-trips but is not the mathematical exponent `PACK` uses, so that target-specific encoding is not adopted here. The runtime uses `frexpf`, doubling the fraction and subtracting one from the exponent because C normalizes to one half instead of one.

Zero satisfies no normalization interval and the references give no shared answer for it. `UNPK` of either zero stores positive zero in `x` and zero in `n`, which round-trips through `PACK` and invents no exponent. `UNPK` of an infinity or a NaN cannot produce a normalized value: it prints `UNPK argument is not finite` and exits.

Both arguments of `PACK` and `UNPK` are resolved left to right and evaluated once. Imported variables stay read-only, so neither procedure can reach one.

### SET

The Report leaves the largest `SET` element implementation-defined. This compiler chooses 31, so a `SET` contains exactly the integers 0 through 31 and bit *n* records membership of element *n*. The empty set is zero and the full set is every bit. A `SET` module variable starts empty because static storage is zero-filled; a `SET` local, like every other local, is uninitialized until assigned.

`ORD` applied to a `SET` reinterprets that bit pattern as a signed `INTEGER` and emits no instruction. `ORD({0})` is 1, `ORD({31})` is `MIN(INTEGER)`, and `ORD(-{})` is -1. Constant folding performs the same reinterpretation, so a folded result and a computed one always agree.

Anything that produces or tests a `SET` element checks that element against 0 through 31 first: a constructor element, either endpoint of a range, the left operand of `IN`, and the second argument of `INCL` and `EXCL`. An element the compiler can fold is a source diagnostic, so `{16 + 16}` fails to compile exactly as `{32}` does. An element that is only known at run time is compared against both bounds before the shift that would otherwise consume it, because QBE reduces a shift count modulo the word width and would silently read 32 as 0. A failed check calls `oberon_set_element_range`, which prints one line and exits.

### CHAR, BYTE, and strings

A `CHAR` is one unsigned byte, so the character set is exactly the 256 ordinals 0 through 255 and the compiler assigns no further meaning to any of them. A quoted string literal denotes the bytes of its source text exactly as they appear in the file: an ASCII file produces one `CHAR` per character, a UTF-8 file produces one `CHAR` per byte of a multi-byte character, and nothing decodes, re-encodes, or validates those bytes. The `nnX` form is a one-character string, not a separate kind of literal, following the Report's `string` production; `0X` is therefore an ordinary one-character string whose character is the null character. `ORD` of a `CHAR` is its ordinal, `CHR` maps an ordinal back, and neither emits an instruction, because a `CHAR` already travels in a word holding its ordinal. `CHR`'s domain is the ordinals 0 through 255: a constant argument outside it is a source diagnostic, and a dynamic one is checked against both bounds and calls `oberon_chr_range` on failure.

`CHAR` and `BYTE` share one byte-wide IR type: both are one unsigned byte in storage, both load zero-extended with `loadub` and store with `storeb`, and both use the word class in every calling position, so a runtime function taking either declares a 32-bit integer parameter in C. Because a loaded value is always 0 through 255, the ordinary signed word comparisons give the correct unsigned ordering for `CHAR` and no new comparison instructions exist. These are the first types whose alignment is one, so a character array global is emitted with alignment one and its exact byte size, an array of `CHAR` is exactly its length in bytes with no padding, and procedure slots keep `alloc4`, QBE's smallest allocation, where over-alignment is harmless.

`BYTE`'s values are the integers 0 through 255, and it is compatible with `INTEGER` as one rule in each direction. Reading a `BYTE` — a variable, an element, a parameter, or a function result — yields an `INTEGER`, so everything downstream is ordinary `INTEGER` behaviour. Writing a `BYTE` checks the range: a value the compiler can fold is a source diagnostic when out of range and free when in range, and any other value is compared against both bounds and calls `oberon_byte_range` on failure. Both reference compilers truncate silently instead; that is declined, because a program that stores 256 into a `BYTE` has no meaning under the Report. `INC` and `DEC` accept a `BYTE` variable through the same checked store, a variable parameter still requires the identical type, and a `FOR` control variable must be `INTEGER` as Report 9.8 demands — which also spares a loop to 255 from trapping on the final increment the source never wrote.

A string is a constant and never a variable's type: it appears only as a literal or as a constant declared from one, and its semantic type carries its character count, which does not include a terminator. A single-character string lowers to a `CHAR` value wherever an expression is lowered, so `ch >= "A"`, `p("A")`, `ch := 0X`, and `RETURN "A"` all follow from one rule; the empty string has no characters and is not a single-character string. A string whose bytes must exist at run time becomes a data object with alignment one holding the characters followed by one null character, each byte written as a decimal item so no escaping rule exists; a string that only ever folds emits nothing. Literals are numbered within their module and never shared, an imported string constant reaches a client as its bytes, and the client emits its own data object from them, so no literal symbol crosses a module boundary.

Assigning a string to a fixed character array requires the character count to be less than the array length. That fit is checked at compile time. Assigning a string through a writable open character array uses the actual's dynamic capacity and checks at run time that the characters and null terminator fit. Both forms copy exactly the characters and one null character and leave the rest of the destination untouched. The relations extend to fixed and open character arrays. Two operands compare when each is a character array or a string and at least one is a character array. The compiler calls `oberon_str_cmp` with both addresses and both bounding lengths. A fixed character array contributes its declared length. An open character array contributes its incoming dynamic length. A string contributes its character count plus its terminator. The walk stops at the first differing pair, at a null present in both, or at the shorter bound. A properly terminated value therefore compares by its characters, an unterminated full array still gets an answer, and nothing reads past an actual length. Two strings fold under the same bounded rule, so a folded comparison and a computed one always agree.

## Structured representation

Arrays and records live in storage and are represented by addresses whenever they cross a procedure boundary. A structured `VAR` parameter is a writable reference. A structured value parameter is a read-only reference, including every field and element selected from it, and no callee-side copy is made. A fixed array formal requires a variable of the identical type. A record value or `VAR` formal also accepts an actual whose static record type extends the formal type. An open-array formal uses the recursive compatibility rule described below. A read-only variable may be passed to either kind of structured value parameter but not to a `VAR` parameter.

A record `VAR` formal expands into a QBE long address followed immediately by a QBE long dynamic descriptor. An ordinary record actual supplies its static descriptor. A forwarded record `VAR` formal supplies its incoming descriptor. A record reached through a checked pointer dereference loads the descriptor from the heap header only when the call needs it. A record value formal still carries only one address because the language does not permit a dynamic type test, guard, or type case on it.

A structured value parameter is an address rather than a local copy because Report 10.1 confines the rule that a value parameter is a local variable holding the actual's value to basic types, and Report 9.1 then forbids assigning to a structured value parameter or to any of its elements. That pairing licenses passing the address and copying nothing, and all three reference compilers read it the same way. The consequence is that aliasing is observable: when one variable is passed both to a value parameter and to a `VAR` parameter of the same call, an assignment through the `VAR` parameter is visible through the value parameter immediately.

A string is not a variable, so it cannot be the actual for a fixed character-array formal, whatever its length. Report 9.1's string exception attaches to an assignment, and no assignment to a structured formal ever happens.

Whole-array assignment and identical-record assignment copy the complete representation through `oberon_copy`, a `memmove` wrapper. Assignment from an extended record to a base record copies exactly the destination type's base prefix. Assignment through a base-record `VAR` formal therefore leaves the actual object's derived tail and dynamic type unchanged. Assignment resolves the destination before the source and accepts self-assignment. Every copied record prefix includes its padding.

### Open array formals

An open array exists only in a formal parameter type. Each `ARRAY OF` prefix creates one semantic open dimension around the qualified base type. An open dimension has no declared length, storage size, alignment, or type identity of its own. Ordinary declarations, fields, pointer bases, procedure results, globals, and slots cannot contain one.

Compatibility consumes one actual array dimension for each open prefix. The consumed actual dimension may be fixed or open and may have any non-negative length. After the prefixes are consumed, the actual must reach the identical qualified base type. This preserves the boundary between `ARRAY OF Row`, where `Row` is one named fixed type, and `ARRAY OF ARRAY OF INTEGER`, where the second dimension is open. It also lets a mixed open and fixed actual be forwarded when both walks reach the same terminal type.

One open-array source parameter expands into one QBE long address followed by one QBE word length for each open prefix. Lengths are ordered from the outermost dimension to the innermost. Each declared name expands independently, even when several names share one parameter section. Procedure semantic types and interfaces keep source signatures for arity and type diagnostics, while the typed IR exposes every expanded machine parameter and call argument.

A resolved place carries the lengths of all array dimensions reachable from it. A fixed dimension contributes an immediate. An open dimension contributes its incoming length temporary. Indexing consumes the current length and leaves the remaining shape on the selected place, so a selected row can be forwarded without resolving its selectors again. Field selection reconstructs the fixed shape of an array field. Pointer dereference preserves read-only status and relies on the existing arbitrary-interior-pointer collector setting when a selected heap address crosses a call.

The stride of an open multidimensional index is a fixed terminal byte size multiplied by every remaining open length before the next fixed type boundary. For `ARRAY OF ARRAY OF INTEGER`, the outer stride is the incoming inner length times four bytes. For `ARRAY OF Row`, the outer stride is the complete fixed size of `Row`. The index check uses the current dynamic length and occurs before widening, stride multiplication, address formation, or evaluation of the next index expression.

Value and `VAR` open arrays have the same machine representation. A value open formal is a borrowed read-only view. A `VAR` open formal is writable. Every field, element, pointer dereference, forwarded actual, modified predefined operation, and nested selection keeps that semantic distinction. A string literal or named string constant may be passed only to a one-dimensional value `ARRAY OF CHAR`; its address and terminator-inclusive length use the same literal object as assignment and comparison.

An open source may be assigned to a fixed array when their immediate element types are identical. The runtime first checks that the source length is no greater than the fixed destination capacity. It then widens the accepted source length, multiplies it by the fixed element size, and copies only that prefix with `oberon_copy`. A larger destination keeps its untouched tail. Assignment into an open destination remains invalid except for the independent string-to-open-character-array rule.

### Array types have identity

Each `ARRAY` constructor written in the source creates one type, and two types are the same type only when they share it. All the names in one declaration share the type that declaration resolved, so `VAR a, b: ARRAY 8 OF INTEGER` declares two variables of one type. Two separately written constructors are different types however alike they look, and a type declaration whose right side is an existing type name is an alias that keeps the existing identity. This is Report 6.2 read as identity rather than shape, and it is what OBNC does. Project Oberon additionally treats two one-dimensional arrays with equal length and the same base type as compatible; that rule is not adopted here, because it would accept separately declared one-dimensional arrays while rejecting the equivalent nested declarations.

`ARRAY N0, N1 OF T` is an abbreviation for `ARRAY N0 OF ARRAY N1 OF T`, so the two spellings build the same descriptors and are interchangeable everywhere, including in the identity rule.

A length is a constant expression that must yield a non-negative `INTEGER`. Zero is an ordinary length.

### Record types have identity and field visibility

Each `RECORD` constructor creates one type, just as each `ARRAY` constructor does. Its semantic descriptor holds its direct base, directly declared fields in declaration order, size, alignment, declared name when it has one, defining module, and internal runtime descriptor symbol. Each field records its type, offset, and export mark. Aliases and imported interfaces share the descriptor, so identity, extension, runtime identity, and field visibility survive module boundaries.

The record extension relation is reflexive and transitive through direct bases. Field lookup starts in the derived record and continues through its bases. An unmarked field is visible only inside the module that declared its owning record. A client sees only marked inherited fields, including when it reads an exported variable whose record type itself is private. A visible inherited field cannot be redeclared. An imported private field is invisible and may be redeclared by the client. Field selection adds the inherited or direct field's fixed byte offset to the base address and carries the base's read-only status through subsequent selectors.

Every record constructor also emits one internal runtime descriptor data object. The object is one aligned pointer-sized word containing its direct base descriptor's address or zero for a root. Internal symbols use the defining module and a module-local ordinal, so anonymous and local records cannot collide and aliases emit no duplicate object. A dynamic test walks these direct-base pointers, using descriptor addresses as nominal type identities. Record comparison is not part of the language.

### Dynamic type operations

`v IS T` accepts a pointer-valued expression when `T` is a pointer type extending the expression's static pointer type. It also accepts a record `VAR` formal when `T` is a record type extending its declared type. The subject is captured once. Pointer tests obtain the dynamic descriptor from a nonnull heap payload, while record tests use the formal's incoming descriptor. A false test yields `FALSE` and does not trap. This compiler defines `NIL IS T` as `FALSE`, including in constant expressions.

A designator guard `v(T)` has the same applicability rules. It tests before any following selector, traps through `oberon_type_guard_failed` when false, and otherwise keeps the same storage while narrowing the designator's static type. Record guards preserve the incoming descriptor. Pointer guards preserve the pointer storage, so a later explicit or implicit dereference still performs the ordinary nil check. A terminal parenthesized postfix remains neutral in the AST until semantic analysis resolves its prefix as either a procedure or an eligible guarded designator.

A type `CASE` accepts one qualified pointer variable or record `VAR` formal. It captures the pointer value or incoming descriptor once, tests one extension type label per arm in source order, and narrows references to the case variable only within the matching arm. The original type is restored before the next arm and after the statement. A nil pointer or other unmatched dynamic type falls through without the scalar `CASE` no-match trap. Scalar `INTEGER` and `CHAR` cases retain their range, overlap, and trap rules.

### Size, alignment, and the target object-size limit

An array occupies exactly its length times its element size, with element *i* at *base + i × element size*. There is no padding between elements and no header. Its alignment is its element type's alignment.

A root record lays out fields in declaration order. Each field starts at the next offset aligned for its type. The record's alignment is the largest field alignment, and its size is rounded up to that alignment so an array of records has the correct stride. An empty root record has size zero and alignment one. A derived record begins with its direct base's complete size, including tail padding. New fields start no earlier than that size, the derived alignment includes the base alignment, and the final size is rounded as usual. Every base is therefore a stable byte-for-byte prefix. Pointer containment is inherited from the base and combined with the new fields.

A zero-length array has size zero and keeps its element alignment. It gains no hidden element and no minimum payload byte: QBE accepts a zero-byte data object and a zero-byte stack allocation, and no valid selector can reach inside one. A zero-length array field reserves no payload but still contributes its alignment.

Layout arithmetic is checked against a target object-size limit of one gibibyte, which is one constant shared by semantic analysis and the driver. It is well below QBE's signed stack-offset range and the reach of the small code model's data references, which leaves room for the code, the runtime's own objects, and linker placement. A single type whose size exceeds it is a source error, and so is a procedure whose locals or a module whose globals cross it in total, reported against the declaration that crossed it. The driver sums the whole program's static data before invoking QBE, so several individually valid modules cannot together produce a link that fails. This is an implementation resource limit, not an Oberon rule.

A module-level structured variable is a zero-filled data object of its complete size and alignment. A procedure-local structured variable reserves its complete size in the activation record and, like every other local, starts uninitialized.

### Indexing is a checked address operation

`a[i, j]` means `a[i][j]`, so each expression in one bracket list is its own index selector. Index expressions are evaluated left to right and exactly once, and each dimension's index is checked against zero inclusive and that dimension's length exclusive before it is widened, scaled, or added to the base address, and before the next dimension's expression runs.

The IR carries the applicable fixed or dynamic length in the index instruction itself rather than leaving the backend to recover it from the base allocation. An inner dimension's base is an address with no allocation of its own, and an open dimension's length exists only as an incoming value. QBE lowering calls `oberon_check_index`, sign-extends the accepted non-negative index, multiplies by the fixed stride factor and any dynamic inner lengths, and then adds the result to the base.

A failed check prints `array index out of bounds` and exits. Like the other runtime failures, it carries no source position yet.

An index the compiler can fold is a source error when it is outside its domain, exactly as an out-of-range `SET` element is. A valid constant index in an executable expression still takes the ordinary checked lowering: there is no optimization pass, and one executable path is what makes the rule literal — every executed index carries its length and checks it before the address is formed. A zero-length array therefore fails every dynamic index without relying on pointer arithmetic or a later load.

Selecting part of a variable does not change whether it can be written. An imported array may be read and indexed, but neither it nor any element or row of it may be assigned or passed where a variable is changed.

### Structured values

Arrays and records are never loaded as scalar values. A structured designator is a whole value only in assignment and parameter passing; arithmetic, conditions, constants, and other scalar contexts reject it. Character arrays additionally participate in the Report's string assignment and comparison rules. Report 10.1 forbids both array and record result types, and records have no equality relation.

### `LEN`

`LEN(v)` is the current array dimension's length as an `INTEGER`. A fixed dimension returns an immediate. An open dimension returns its incoming length value. The argument designator is still resolved, so `LEN(a[f()])` calls `f` once and checks its result before answering with either the selected dynamic length or a statically known inner length. Nothing about a source effect or an invalid selection is erased by the answer being known.

In a required constant context, such as a constant declaration or another array's length, `LEN` follows the designator's types without executing its selectors. It folds only when the selected dimension is fixed. An open dimension is not constant. A fixed dimension reached after selecting through an open prefix can still fold because its length does not depend on the actual. The array variable itself need not be a constant, and a dynamic INTEGER index does not prevent folding a fixed selected dimension. Every index is still type-checked. An out-of-range constant index remains a source error only when the applicable bound is fixed. Pointer selectors are followed without loading or checking the pointer.

Nothing observable is skipped, because the two things this path declines to do are both unobservable. Reading a variable has no effect, so an index that names one can be ignored. Calling a procedure does have an effect, so an index that contains a call is a source error instead — the same error a call anywhere else in a constant expression gets, and the reason Report 8 can define a constant expression as one a textual scan evaluates without running the program.

### Types across module boundaries

A module's interface carries its exported type names alongside its constants, variables, and procedures. A client reaches one through the same qualified lookup as any other member, and a private type name is simply absent. Cloning an interface clones shared type handles rather than rebuilding types, so the original name, a re-exported alias, and every client that imports either all denote one type, and an assignment between variables declared through different names of it is an assignment between identical types.

An exported variable may have a private or inline structured type. Its interface carries the type needed to read and select from the variable without giving the client a name to declare another variable of that type, so the type stays private while the variable stays usable. A record descriptor also carries its defining module, which lets field lookup hide unmarked fields in every client without rebuilding the type.

## Backend

The backend targets QBE.

QBE provides:

* instruction selection
* register allocation
* calling conventions
* native code generation

The compiler is responsible for language semantics, object layout, runtime checks, and lowering to QBE IL.

## Runtime

A small C runtime provides:

* memory allocation
* module initialization
* runtime checks, including the array index check
* the nil-procedure call check
* open-array assignment capacity checks
* whole-value copying
* basic runtime support
* standard library implementation

The runtime intentionally remains minimal.

### Heap objects and allocation

`NEW(p)` passes the payload size and the currently selected bound record's static descriptor to an allocation wrapper. The allocation contains one hidden pointer-sized descriptor header immediately before the source payload, while the returned pointer still addresses the first source field. Source field offsets, record sizes, array strides, and copy counts therefore exclude the header. Allocation failure returns null before any header write or pointer adjustment and is stored as `NIL`. A zero-size payload reserves one additional hidden byte so the returned interior pointer is nonnull, lies within its allocation, and remains distinct from other live empty objects. The generated program calls only `oberon_alloc` and `oberon_alloc_atomic`, never BDWGC directly.

Each record layout records whether its complete source payload contains a data pointer. A pointer field makes the record scanned without following the pointer's base. A procedure field does not. A derived record inherits pointer containment from its base. A positive-length array inherits the property from its element, a zero-length array does not, and a nested record contributes its already computed property. `NEW` selects `oberon_alloc` for a pointer-containing payload and `oberon_alloc_atomic` otherwise. A descriptor pointer in an atomic object's header names static data rather than a GC allocation and does not require scanning.

Pointer globals, stack slots, parameters, temporaries, and scanned heap fields are ordinary conservative roots. Every returned payload pointer is an interior pointer because of the descriptor header. A selected `NEW` target may also leave only an interior field address live across the allocation call, so `oberon_init` enables arbitrary interior-pointer recognition before `GC_INIT`. This preserves both heap objects and source evaluation order because every target selector is evaluated once before allocation.

## Garbage Collection

Heap allocation uses the Boehm–Demers–Weiser conservative garbage collector (BDWGC).

Generated code allocates through runtime wrappers rather than calling BDWGC directly:

* `oberon_alloc`
* `oberon_alloc_atomic`

This isolates the compiler from the underlying allocator and allows the GC implementation to be replaced in the future if desired.
