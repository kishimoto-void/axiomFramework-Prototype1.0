#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
cat axiom_mas_v3.2.2_AbsoluteAnchor.go.split.* > axiom_mas_v3.2.2_AbsoluteAnchor.go
echo "joined $(wc -c < axiom_mas_v3.2.2_AbsoluteAnchor.go) bytes"
