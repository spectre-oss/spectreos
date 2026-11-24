#!/usr/bin/env python3
"""delta update hook: validate and save decisions before acknowledging them."""
import json
import os
from pathlib import Path
import sys
from bhoo_review.store import DEFAULT_LEDGER, process_event

if __name__ == '__main__':
    event = json.load(sys.stdin)
    reply = process_event(event, Path(os.environ.get('BHOO_REVIEW_LEDGER', DEFAULT_LEDGER)))
    if reply:
        print(json.dumps(reply, ensure_ascii=False))
