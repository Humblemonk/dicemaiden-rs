#!/bin/sh
# Check the remaining Discord session start quota for today.
#
# Intended to be run by hand against a live deployment:
#   kubectl exec deploy/dicemaiden-rs -- quota.sh
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
