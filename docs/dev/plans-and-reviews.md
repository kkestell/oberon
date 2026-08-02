# Plans and Reviews

Development happens in slices, and the plan for a slice and the review of the result are both worth keeping. A plan records what was intended and what was deliberately left out; a review records what was found and why it mattered. Together they explain choices that the code and the commit log leave implicit.

Write them into the repository:

[docs/dev/plans/](plans/)
: One file per implementation plan, written before the work starts.

[docs/dev/reviews/](reviews/)
: One file per code review, written after a slice lands.

## Naming

`YYYY-MM-DD-NNN-short-description.md`, for example `2026-08-02-001-end-to-end-qbe-libgc-integration.md`.

The date is the day the document is written. The sequence number counts within its own directory, not across both, and starts at `001`. Check the directory for the highest existing number before picking the next one — two documents written on the same day get consecutive numbers, and the sequence does not reset each day. The description is a few kebab-case words naming the slice or the review subject; avoid the word "plan" or "review" in it, since the directory already says which it is.

## Writing them

Do this automatically, without being asked. Write the plan file as part of agreeing on a plan, and the review file as part of delivering a review — not as a separate chore afterwards.

Both are ordinary prose documents, so the rules in [English, Please](../../AGENTS.md#english-please) apply to them in full. A few specifics:

* Reviews pin the state they were written against — a commit, or the fact that the tree was uncommitted. Findings outlive line numbers, so name functions rather than citing line numbers that will rot.
* A finding confirmed by compiling and running something records the program and its actual output. That is what makes it checkable a year later.
* Record what was deliberately declined and why, in plans and reviews alike. The rejected option is often the more useful half of the document.
