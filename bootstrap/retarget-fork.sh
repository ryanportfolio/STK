#!/bin/bash
# retarget-fork.sh - point a fork of claude-starter at its own repo id.
#
# Rewrites every functional upstream reference in one pass: the template repo
# id in the sync-starter/init-project skills, the drift-check URL in
# session-start.sh, the bootstrap defaults, and the plugin manifests. The
# current id is read from .claude-plugin/plugin.json ("repository"), so the
# script works again if you re-fork or rename later.
#
# LICENSE is intentionally untouched: the original copyright attribution
# stays; add your own line above it if you wish.
#
# Usage: bash bootstrap/retarget-fork.sh <owner>/<repo>
# Then review 'git diff' and commit.

set -euo pipefail

NEW="${1:-}"
case "$NEW" in
  ?*/?*) ;;
  *) echo "Usage: bash bootstrap/retarget-fork.sh <owner>/<repo>" >&2; exit 1 ;;
esac
NEW_OWNER="${NEW%%/*}"

cd "$(git rev-parse --show-toplevel)"

OLD=$(sed -n 's|.*"repository": *"https://github.com/\([^"]*\)".*|\1|p' .claude-plugin/plugin.json)
if [ -z "$OLD" ]; then
  echo "Could not read the current repo id from .claude-plugin/plugin.json" >&2
  exit 1
fi
OLD_OWNER="${OLD%%/*}"

if [ "$OLD" = "$NEW" ]; then
  echo "Already targeting $NEW - nothing to do."
  exit 0
fi

FILES=$(git grep -l "$OLD_OWNER" -- ':(exclude)LICENSE' ':(exclude)bootstrap/retarget-fork.sh' || true)
if [ -z "$FILES" ]; then
  echo "No references to $OLD_OWNER found - nothing to do."
  exit 0
fi

# perl -pi, not sed -i: BSD sed (macOS) and GNU sed disagree on -i syntax.
# Order matters: full repo id first, then owner-only leftovers (plugin
# author url, marketplace owner name).
for f in $FILES; do
  perl -pi -e "s|\Q$OLD\E|$NEW|g; s|github\.com/\Q$OLD_OWNER\E|github.com/$NEW_OWNER|g; s|\"\Q$OLD_OWNER\E\"|\"$NEW_OWNER\"|g" "$f"
  echo "retargeted: $f"
done

LEFT=$(git grep -l "$OLD_OWNER" -- ':(exclude)LICENSE' ':(exclude)bootstrap/retarget-fork.sh' || true)
echo ""
if [ -n "$LEFT" ]; then
  echo "WARNING: references to $OLD_OWNER remain in:"
  echo "$LEFT"
  echo "Inspect these by hand."
else
  echo "Done. All functional references now point at $NEW."
fi
echo "LICENSE attribution left untouched. Review with 'git diff', then commit."
