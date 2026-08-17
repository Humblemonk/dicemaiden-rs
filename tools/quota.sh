#!/bin/sh
# Check the remaining Discord session start quota for today.
#
# Intended to be run by hand against a live deployment:
#   kubectl exec deploy/dicemaiden-rs -- quota.sh
set -eu

if [ -f /app/.env ]; then
	set -a
	# shellcheck disable=SC1091
	. /app/.env
	set +a
fi

: "${DISCORD_TOKEN:?DISCORD_TOKEN not set}"

curl -sf -H "Authorization: Bot ${DISCORD_TOKEN}" \
	https://discord.com/api/v10/gateway/bot |
	jq -r '
        "recommended shards : \(.shards)",
        "sessions remaining : \(.session_start_limit.remaining)/\(.session_start_limit.total)",
        "max concurrency    : \(.session_start_limit.max_concurrency)",
        "quota resets in    : \(.session_start_limit.reset_after / 3600000 | floor)h",
        (if .session_start_limit.remaining < (.session_start_limit.total / 4)
         then "\nWARNING: under 25% of the daily session allowance remains"
         else empty end)
    '
