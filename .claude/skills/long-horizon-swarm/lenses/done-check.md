# Lens: done-check

**Question:** can this round's done-check pass while the step is still wrong?

Use on the draft step and done-check, before the host takes the Baseline and freezes them.
After the freeze a finding goes to the next round's Plan; the frozen check is not edited, so
a weak one found late costs a whole round.

## Inspect

- The Current round block: Step, Done-check, Write scope.
- The contract's Acceptance list and the Dead ends.
- The code or files the done-check actually exercises. Read them; do not trust the check's
  name.

## A finding is

- **False positive:** a concrete wrong implementation that would still pass, e.g. a stub
  returning the expected value, a test that never reaches the changed path, a check that
  reads a file the executor can write, or part of the step's acceptance that no command
  verifies.
- **Unrunnable:** a check that cannot run as written: wrong cwd, missing fixture, command
  absent on this machine.

"Could be stronger" without a passing wrong implementation or a failing command is not a
finding.

## Return

One block per finding, in the format for its kind:

- **False positive:** the wrong implementation, the done-check command that passes on it,
  and a tightened check that fails on it.
- **Unrunnable:** the exact command, the cwd you ran it from, its failure output, and a
  corrected command.

End with `no findings` when none survive, so the host can end the cycle.
