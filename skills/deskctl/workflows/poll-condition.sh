#!/usr/bin/env bash
# poll-condition.sh - poll the desktop until a condition is met
# usage: ./poll-condition.sh <match-string> [interval-seconds] [max-attempts]
# example: ./poll-condition.sh "Tickets Available" 5 60
# example: ./poll-condition.sh "Order Confirmed" 3 20
# example: ./poll-condition.sh "Download Complete" 10 30
#
# checks window titles for the match string every N seconds.
# exits 0 when found, exits 1 after max attempts.
set -euo pipefail

MATCH="${1:?usage: poll-condition.sh <match-string> [interval] [max-attempts]}"
INTERVAL="${2:-5}"
MAX="${3:-60}"

attempt=0
while [ "$attempt" -lt "$MAX" ]; do
  attempt=$((attempt + 1))

  # snapshot and check window titles
  windows=$(deskctl list-windows --json 2>/dev/null || echo '{"success":false}')
  if echo "$windows" | grep -qi "$MATCH"; then
    echo "FOUND: '$MATCH' detected on attempt $attempt"
    deskctl snapshot --annotate
    exit 0
  fi

  # also check screenshot text via active window title
  active=$(deskctl get active-window --json 2>/dev/null || echo '{}')
  if echo "$active" | grep -qi "$MATCH"; then
    echo "FOUND: '$MATCH' in active window on attempt $attempt"
    deskctl snapshot --annotate
    exit 0
  fi

  echo "attempt $attempt/$MAX - '$MATCH' not found, waiting ${INTERVAL}s..."
  sleep "$INTERVAL"
done

echo "NOT FOUND: '$MATCH' after $MAX attempts"
deskctl snapshot --annotate
exit 1
