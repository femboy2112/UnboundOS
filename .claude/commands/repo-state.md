Spawn the `campaign-state` agent and report its verdict plus punch
list verbatim.

Read-only diagnostic. Do not run mission steps based on this output
without an explicit `/go`. If the verdict is `STOP` or
`USER-JUDGMENT`, advise the operator against `/go` until the drift is
resolved.

Also run `make repo-state` and surface its JSON output alongside the
agent verdict so the operator can compare.

Never commit, never push, never edit files from this command.
