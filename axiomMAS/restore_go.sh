#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
cat axiom_mas_v3.2.2_AbsoluteAnchor.go.b64.part* > /tmp/_mas_b64_join
base64 -d /tmp/_mas_b64_join | gzip -d > axiom_mas_v3.2.2_AbsoluteAnchor.go
rm -f /tmp/_mas_b64_join
echo "OK: axiom_mas_v3.2.2_AbsoluteAnchor.go restored ($(wc -c < axiom_mas_v3.2.2_AbsoluteAnchor.go) bytes)"
