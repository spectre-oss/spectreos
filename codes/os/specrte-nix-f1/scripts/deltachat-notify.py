#!/usr/bin/env python3
"""Durable, owner-only DeltaChat notifications using the existing bot account."""
import argparse
from contextlib import closing
import fcntl
import json
import os
from pathlib import Path
import sqlite3
import sys
import time

DEFAULT_BOT = Path(os.environ.get('SPECTRE_DELTABOT_DIR', str(Path.home() / 'deltabot/codex-deltachat')))


def database(bot):
    db = sqlite3.connect(bot / '.data' / 'notifications.sqlite3', timeout=30)
    db.execute('CREATE TABLE IF NOT EXISTS notices (key TEXT PRIMARY KEY, text TEXT NOT NULL, message INTEGER, state TEXT NOT NULL DEFAULT "pending", error TEXT)')
    return db


def enqueue(db, key, text):
    if not text.strip() or len(text) > 3500:
        raise ValueError('Notification must contain 1–3500 characters')
    db.execute('INSERT OR IGNORE INTO notices(key,text) VALUES (?,?)', (key, text))
    db.commit()


def deliver(bot, db):
    """Share the account lock with main.py; never open its database concurrently."""
    if not db.execute('SELECT 1 FROM notices WHERE state != "delivered"').fetchone():
        return
    sys.path.insert(0, str(bot))
    from src.deltachat.rpc import Core
    from src.bot.config import load
    data = bot / '.data'
    with (data / 'bot.lock').open('a') as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            return
        settings = load(data / 'settings.json')
        pairing = json.loads((data / 'onboarding.json').read_text())
        account, contact, chat_id = (pairing[x] for x in ('account', 'contact', 'chat'))
        if contact not in settings.get('owner_ids', []):
            raise ValueError('Paired contact is not a pinned owner')
        core = Core(settings.get('core', str(bot / 'bin/deltachat-rpc-server')), data / 'accounts')
        try:
            person = core.call('get_contact', account, contact)
            chat = core.call('get_full_chat_by_id', account, chat_id)
            owners = {x.casefold() for x in settings.get('owners', [])}
            if (person.get('address', '').casefold() not in owners or
                not person.get('e2eeAvail') or not chat.get('isEncrypted') or
                chat.get('chatType') != 'Single' or contact not in chat.get('contactIds', [])):
                raise ValueError('Recipient is not the paired encrypted owner chat')
            core.call('start_io', account)
            for key, text, message in db.execute('SELECT key,text,message FROM notices WHERE state != "delivered"').fetchall():
                if message is None:
                    message = core.call('misc_send_text_message', account, chat_id, text)
                    db.execute('UPDATE notices SET message=?,state="queued",error=NULL WHERE key=?', (message, key))
                    db.commit()
                deadline = time.monotonic() + 45
                while time.monotonic() < deadline:
                    status = core.call('get_message', account, message).get('state')
                    if status in (26, 28):
                        db.execute('UPDATE notices SET state="delivered",error=NULL WHERE key=?', (key,))
                        db.commit()
                        break
                    if status == 24:
                        # Core explicitly reports failure. Re-enqueue on a later pass.
                        db.execute('UPDATE notices SET message=NULL,state="pending",error="Core delivery failed" WHERE key=?', (key,))
                        db.commit()
                        break
                    time.sleep(1)
        finally:
            core.close()


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bot-dir', type=Path, default=DEFAULT_BOT)
    parser.add_argument('--worker', action='store_true')
    parser.add_argument('--status', action='store_true')
    parser.add_argument('--key', help='Stable event ID for deduplication')
    parser.add_argument('text', nargs='?')
    args = parser.parse_args()
    with closing(database(args.bot_dir)) as db:
        if args.status:
            print(json.dumps(db.execute('SELECT key,state,error FROM notices').fetchall()))
        elif args.worker:
            with (args.bot_dir / '.data' / 'notification-worker.lock').open('a') as lock:
                fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                while True:
                    try:
                        deliver(args.bot_dir, db)
                    except Exception as error:
                        print(f'Notification retry: {type(error).__name__}: {error}', flush=True)
                    time.sleep(30)
        else:
            if not args.key or not args.text:
                parser.error('--key and text are required')
            enqueue(db, args.key, args.text)
            print('Queued', args.key)


if __name__ == '__main__':
    main()
