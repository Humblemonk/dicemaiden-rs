#!/bin/sh
# Check the remaining Discord session start quota for today.
#
# Intended to be run by hand against a live deployment:
#   kubectl exec deploy/dicemaiden-rs -- quota.sh
set -eu

# shellcheck source=tools/dicemaiden-env.sh
# shellcheck disable=SC1091
. "$(dirname "$0")/dicemaiden-env.sh"
load_env /app/.env

: "${DISCORD_TOKEN:?DISCORD_TOKEN not set}"

# Captured rather than piped straight into jq: a pipeline reports the exit status of
# its last command, so a failed curl would otherwise leave the script exiting 0 with no
# output. --proto pins https and no -L means the Authorization header cannot follow a
# redirect to another host.
if ! response=$(curl -sf --proto '=https' --connect-timeout 10 --max-time 30 \
	-H "Authorization: Bot ${DISCORD_TOKEN}" \
	https://discord.com/api/v10/gateway/bot); then
	echo "request to discord.com/api/v10/gateway/bot failed" >&2
	exit 1
fi

printf '%s' "$response" |
	jq -r '
        "recommended shards : \(.shards)",
        "sessions remaining : \(.session_start_limit.remaining)/\(.session_start_limit.total)",
        "max concurrency    : \(.session_start_limit.max_concurrency)",
        "quota resets in    : \(.session_start_limit.reset_after / 3600000 | floor)h",
        (if .session_start_limit.remaining < (.session_start_limit.total / 4)
         then "\nWARNING: under 25% of the daily session allowance remains"
         else empty end)
    '
