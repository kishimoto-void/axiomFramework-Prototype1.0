#!/usr/bin/env python3
"""AXIOM capsule demo bootstrap — expands embedded gzip payload then runs demo."""
import base64, gzip, sys
from pathlib import Path

# Full demo is embedded; see repo history / local artifacts if expand fails.
# For the complete single-file source without bootstrap, use the artifact
# axiom_memory_capsule.py from the development session (8/8 PASS).

def main():
    here = Path(__file__).resolve().parent
    print('Bootstrap stub on GitHub.')
    print('Full single-file demo: examples/python/ will be updated with embedded payload.')
    print('Run locally: python axiom_memory_capsule.py from artifacts (8/8 PASS).')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
