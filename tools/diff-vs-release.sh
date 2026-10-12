#!/usr/bin/env bash
# Compare every roll result in the working tree against a release.
#
#   tools/diff-vs-release.sh            # against the latest v* tag
#   tools/diff-vs-release.sh v1.8.0     # against a given tag or commit
#
# Builds the release in a temporary worktree, rolls the same expressions in both
# (the corpus, the corpus with math around it, and a grid of modifier families;
# see tests/release_diff.rs) at fixed seeds, and lists what changed, most
# dangerous first. Exits 1 if any roll that worked in the release now gives a
# different result: explain every one before shipping.
#
# A change in RNG draw order makes every random roll differ. If that floods the
# report, check tests/snapshots/deterministic_rolls.snap instead: it is immune.
set -euo pipefail

repo=$(git rev-parse --show-toplevel)
release=${1:-$(git -C "$repo" describe --tags --abbrev=0 --match 'v*')}
work=$(mktemp -d)
trap 'git -C "$repo" worktree remove --force "$work/release" >/dev/null 2>&1; rm -rf "$work"' EXIT

git -C "$repo" worktree add -q --detach "$work/release" "$release"
cp "$repo/tests/release_diff.rs" "$work/release/tests/"

export RELEASE_DIFF_CORPUS="$repo/tests/corpus/expressions.txt"
dump() { # dump <tree> <output> [target dir]
	(cd "$1" && RELEASE_DIFF_OUT="$2" CARGO_TARGET_DIR="${3:-$1/target}" \
		cargo test -q --test release_diff -- --ignored >/dev/null)
}
echo "Rolling with $release..." >&2
dump "$work/release" "$work/old.tsv" "$repo/target/release-diff"
echo "Rolling with the working tree..." >&2
dump "$repo" "$work/new.tsv"

# Fields: expression, seed, OK|ERR, output. Expressions are grouped by shape
# (numbers as N, + - * / as op), so one change repeated across the corpus and
# its leading/trailing variants reads as one line
# with a count; the example shown is the first expression of that shape.
awk -F'\t' -v release="$release" '
    NR == FNR { old[$1 FS $2] = $3 FS $4; next }
    {
        key = $1 FS $2
        if (!(key in old) || old[key] == $3 FS $4 || seen[$1]++) next
        split(old[key], was, FS)
        if (was[1] == "OK" && $3 == "OK")        group = 1
        else if (was[1] == "OK")                 group = 2
        else if ($3 == "OK")                     group = 3
        else                                     group = 4
        total[group]++
        shape = $1
        gsub(/[0-9]+/, "N", shape)
        gsub(/ *[-+*\/] */, " op ", shape)
        if (!((group, shape) in count)) {
            order[group, ++shapes[group]] = shape
            example[group, shape] = sprintf("  e.g. %s\n    %s: %s\n    now:    %s\n", $1, release, was[2], $4)
        }
        count[group, shape]++
    }
    END {
        title[1] = "CHANGED RESULTS: worked in " release ", different now. Explain every one"
        title[2] = "NOW AN ERROR: worked in " release
        title[3] = "NOW WORKS: was an error in " release ". Check the new results are right"
        title[4] = "ERROR TEXT CHANGED"
        for (g = 1; g <= 4; g++) {
            if (!total[g]) continue
            printf "\n== %s: %d expressions, %d shapes ==\n", title[g], total[g], shapes[g]
            for (i = 1; i <= shapes[g]; i++) {
                shape = order[g, i]
                printf "%5d  %s\n%s", count[g, shape], shape, example[g, shape]
            }
        }
        printf "\nSummary vs %s: %d changed, %d now error, %d now work, %d error text\n",
            release, total[1], total[2], total[3], total[4]
        exit (total[1] > 0)
    }
' "$work/old.tsv" "$work/new.tsv"
