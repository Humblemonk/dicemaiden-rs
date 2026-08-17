#!/bin/sh
# Submit (or preview, with --dry-run) the current server count to top.gg.
#
# Intended to be run by hand against a live deployment:
#   kubectl exec deploy/dicemaiden-rs -- topgg.sh --dry-run
set -eu

# Read /app/.env without executing it. Sourcing is unsafe here: a value containing
# spaces or shell metacharacters gets run as a command, and the bot's dotenv parser
# accepts lines that /bin/sh does not. Variables already set in the environment win,
# matching dotenv and Kubernetes deployments where values come from a Secret.
load_env() {
	[ -f "$1" ] || return 0
	while IFS= read -r line || [ -n "$line" ]; do
		line=${line%$CR}
		line=${line#"${line%%[![:space:]]*}"}
		case $line in
		'' | '#'*) continue ;;
		esac
		line=${line#export }
		case $line in
		*=*) ;;
		*) continue ;;
		esac
		key=${line%%=*}
		value=${line#*=}
		key=${key%"${key##*[![:space:]]}"}
		value=${value#"${value%%[![:space:]]*}"}
		case $key in
		'' | *[!A-Za-z0-9_]* | [0-9]*) continue ;;
		esac
		case $value in
		\"*\") value=${value#\"} value=${value%\"} ;;
		\'*\') value=${value#\'} value=${value%\'} ;;
		esac
		if env | grep -q "^${key}="; then
			continue
		fi
		export "${key}=${value}"
	done <"$1"
}

CR=$(printf '\r')
load_env /app/.env

# Your bot's Application ID from the Discord developer portal — the same number
# that appears in your bot's top.gg URL.
: "${TOPGG_BOT_ID:?TOPGG_BOT_ID not set}"

# DATABASE_URL is a sqlx URL (sqlite:data/main.db, sqlite:/app/data/main.db,
# sqlite:///app/data/main.db). Relative paths resolve against /app, which is both
# the bot's WORKDIR and the volume mount point in production.
if [ -n "${DATABASE_URL:-}" ]; then
	db=${DATABASE_URL#sqlite:}
	db=${db#//}
	db=${db%%\?*}
	case "$db" in
	/*) ;;
	*) db=/app/$db ;;
	esac
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

# API is the legacy name for this token, kept so existing deployments keep working.
token=${TOPGG_TOKEN:-${API:-}}
if [ -z "$token" ]; then
	echo "neither TOPGG_TOKEN nor API is set" >&2
	exit 1
fi

# --proto pins https; no -L, so the Authorization header cannot follow a redirect.
response=$(curl -s -w '\n%{http_code}' -X POST \
	--proto '=https' --connect-timeout 10 --max-time 30 \
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
	if [ -n "$body" ]; then
		printf '%s\n' "$body" >&2
	fi
	exit 1
fi
