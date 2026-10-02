# Runtime failure positions and output ordering

## Plan

`agents/plans/2026-10-02-021-runtime-failure-positions.md`

## Summary

Every runtime check and trap now carries an `ir::Site`, and the runtime flushes stdout and prints `Module:line:column: message` before exiting with status 1. The plan's goal is met.

## Decisions

- A string copied into an open array reports the position `copy_string` already receives, the string expression, because that is where the folded diagnostic for a fixed destination points. The open-array prefix copy reports the assignment statement, as the plan lists.
- A `BYTE` value parameter and a `BYTE` `RETURN` report the stored expression, the same as a `BYTE` assignment.
- The field selector position the parser records is the field name, so `p.next.n` with a nil `p.next` reports the column of `n`.

## Checks run

- `make test` — 43 unit tests and 6 integration tests pass, including `failure_follows_earlier_output` and all sixty regenerated `tests/failures` cases.
- `cargo fmt --check` — Clean after `cargo fmt`.
- `cargo clippy --all-targets -- -D warnings` — Clean.

## Manual verification

1. Each regenerated `.expected` position points at the construct named in the plan. The loop prints each failure with its source line and a caret under the reported column; library failures resolve to `lib/`.

   ```sh
   for e in $(find tests/failures -name '*.expected' | sort); do l=$(cat $e); mod=${l%%:*}; rest=${l#*:}; line=${rest%%:*}; rest=${rest#*:}; col=${rest%%:*}; f=$(dirname $e)/$mod.Mod; [ -f $f ] || f=lib/$mod.Mod; echo "$l"; sed -n ${line}p $f; printf '%*s^\n' $((col-1)) ''; done
   ```

   Every caret sits on the index expression, `^` or field name, call, `ASSERT`, operator, selector, guard, or assignment the plan's "Which position" names.
