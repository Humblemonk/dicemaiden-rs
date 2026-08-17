#!/bin/sh
# Submit (or preview, with --dry-run) the current server count to top.gg.
#
# Intended to be run by hand against a live deployment:
#   kubectl exec deploy/dicemaiden -- /app/tools/topgg.sh --dry-run
set -eu

if [ -f /app/.env ]; then
	set -a
	# shellcheck disable=SC1091
	. /app/.env
	set +a
fi

# Your bot's Application ID from the Discord developer portal — the same number
# that appears in your bot's top.gg URL.
: "${TOPGG_BOT_ID:?TOPGG_BOT_ID not set}"

# DATABASE_URL is a sqlx URL (sqlite:/app/main.db or sqlite:///app/main.db).
if [ -n "${DATABASE_URL:-}" ]; then
	db=${DATABASE_URL#sqlite:}
	db=${db#//}
	db=${db%%\?*}
else
	db=/app/main.db
fi

if [ ! -f "$db" ]; then
	echo "database not found at $db" >&2
	exit 1
fi

# -readonly avoids creating an empty database on a bad path and avoids taking a
# write lock while the bot is running; .timeout matches the old Ruby busy_timeout.
query() {
	sqlite3 -readonly -cmd '.timeout 10000' "$db" "$@"
}

query -column -header \
	'SELECT process_id, shard_start, shard_count, server_count, timestamp
     FROM process_stats ORDER BY shard_start;'

servers=$(query 'SELECT COALESCE(SUM(server_count), 0) FROM process_stats;')
shards=${TOTAL_SHARDS:-$(query 'SELECT COALESCE(MAX(total_shards), 0) FROM process_stats;')}

echo
echo "total: ${servers} servers across ${shards} shards"

if [ "$servers" -eq 0 ]; then
	echo "refusing to submit a count of zero — has any process reported in?" >&2
	exit 1
fi

if [ "${1:-}" = "--dry-run" ]; then
	echo "--dry-run: not submitting"
	exit 0
fi

token=${TOPGG_TOKEN:-${API:?TOPGG_TOKEN not set}}

response=$(curl -s -w '\n%{http_code}' -X POST \
	-H "Authorization: ${token}" \
	-H 'Content-Type: application/json' \
	-d "{\"shard_count\": ${shards}, \"server_count\": ${servers}}" \
	"https://top.gg/api/bots/${TOPGG_BOT_ID}/stats")

# The status code is appended on its own trailing line by -w above.
code=$(printf '%s' "$response" | tail -n 1)
body=$(printf '%s' "$response" | sed '$d')

if [ "$code" = "200" ]; then
	echo "submitted"
else
	echo "top.gg rejected the update: HTTP ${code}" >&2
	[ -n "$body" ] && echo "$body" >&2
	exit 1
fi
