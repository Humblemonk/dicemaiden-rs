#!/bin/sh
# Shared .env loading for the spot-check scripts. Intended to be sourced, not run;
# the shebang and executable bit exist to satisfy super-linter's BASH_EXEC check.
# Executing it directly defines load_env in a subshell and exits, doing nothing.
#
# Read a KEY=VALUE file without executing it. Sourcing an .env is unsafe: a value
# containing spaces or shell metacharacters gets run as a command, and the bot's dotenv
# parser accepts lines that /bin/sh does not. Variables already set in the environment
# win, matching dotenv and Kubernetes deployments where values come from a Secret.

CR=$(printf '\r')

load_env() {
	[ -f "$1" ] || return 0
	while IFS= read -r line || [ -n "$line" ]; do
		line=${line%"$CR"}
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
