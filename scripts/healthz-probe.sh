#!/usr/bin/env bash
# Probe a kiryuu /healthz endpoint on a fixed interval and append timing rows to a CSV log.
#
# Usage:
#   ./scripts/healthz-probe.sh
#   KIRYUU_HEALTHZ_URL=http://127.0.0.1:6969/healthz ./scripts/healthz-probe.sh
#   KIRYUU_PROBE_INTERVAL=120 KIRYUU_PROBE_LOG=./healthz-probe.csv ./scripts/healthz-probe.sh
#
# Stop with Ctrl-C. Safe to run under systemd, tmux, or nohup.

set -euo pipefail

URL="${KIRYUU_HEALTHZ_URL:-http://tracker.mywaifu.best:6969/healthz}"
INTERVAL="${KIRYUU_PROBE_INTERVAL:-120}"
LOG="${KIRYUU_PROBE_LOG:-./healthz-probe.csv}"
CONNECT_TIMEOUT="${KIRYUU_PROBE_CONNECT_TIMEOUT:-10}"
MAX_TIME="${KIRYUU_PROBE_MAX_TIME:-15}"

mkdir -p "$(dirname "$LOG")"

if [[ ! -f "$LOG" ]]; then
	printf '%s\n' \
		'timestamp_utc,http_code,ok,time_namelookup_s,time_connect_s,time_appconnect_s,time_starttransfer_s,time_total_s,error,active_requests,torrents' \
		>"$LOG"
fi

probe_once() {
	local ts body tmp http_code curl_exit ok error active torrents
	ts="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
	tmp="$(mktemp)"
	body="$(mktemp)"

	set +e
	http_code="$(
		curl -sS \
			--connect-timeout "$CONNECT_TIMEOUT" \
			--max-time "$MAX_TIME" \
			-o "$body" \
			-w '%{http_code} %{time_namelookup} %{time_connect} %{time_appconnect} %{time_starttransfer} %{time_total}' \
			"$URL" 2>"$tmp"
	)"
	curl_exit=$?
	set -e

	if [[ $curl_exit -ne 0 ]]; then
		ok=0
		error="$(tr '\n' ' ' <"$tmp" | sed 's/"/""/g')"
		http_code=000
		printf '%s,000,0,,,,,%.6f,"%s",,\n' \
			"$ts" "$MAX_TIME" "$error" >>"$LOG"
		rm -f "$tmp" "$body"
		return
	fi

	read -r http_code time_namelookup time_connect time_appconnect time_starttransfer time_total <<<"$http_code"

	if [[ "$http_code" == "200" ]]; then
		ok=1
		error=""
	else
		ok=0
		error="unexpected http $http_code"
	fi

	active="$(grep -E '^active_requests=' "$body" 2>/dev/null | cut -d= -f2 || true)"
	torrents="$(grep -E '^torrents=' "$body" 2>/dev/null | cut -d= -f2 || true)"

	printf '%s,%s,%s,%.6f,%.6f,%.6f,%.6f,%.6f,"%s",%s,%s\n' \
		"$ts" "$http_code" "$ok" \
		"$time_namelookup" "$time_connect" "$time_appconnect" \
		"$time_starttransfer" "$time_total" "$error" \
		"${active:-}" "${torrents:-}" >>"$LOG"

	rm -f "$tmp" "$body"
}

trap 'echo "stopped; log: $LOG" >&2; exit 0' INT TERM

echo "probing $URL every ${INTERVAL}s -> $LOG" >&2

while true; do
	probe_once
	sleep "$INTERVAL"
done
