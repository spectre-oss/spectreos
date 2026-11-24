#!/usr/bin/env python3
"""Index and test this workspace's documentation through ~/llmcli."""
import argparse
import json
from pathlib import Path
import subprocess
import sys

CODE = Path(__file__).resolve().parents[1]
WORKSPACE = CODE.parents[1]
STORE = CODE / '.code-sync/docs-vector'
PROJECT = 'poco-f1-docs'
SOURCES = ['README.md', 'docs', 'codes/spectre-nix/README.md',
           'codes/spectre-nix/STATUS.md', 'codes/spectre-nix/infra/README.md',
           'codes/spectre-nix/keyboard/README.md',
           'codes/spectre-nix/keyboard/typing/README.md',
           'codes/spectre-nix/scripts/INSTALL.md',
           'codes/spectre-nix/scripts/INSTALL-BOOT.md']
CASES = [
    ('What terminal activation mismatch caused the Phosh compositor startup loop?',
     ['phosh.md', 'current-system.md', 'inspection-2026-10-20.md'], ['tty1', 'tty2']),
    ('Which machine is required to compile and build this phone system?',
     ['README.md', 'current-system.md'], ['build host']),
    ('What computer and hardware are described in the historical my_system notes?',
     ['my_system.txt.md', 'history.md', 'file-catalog.md'], ['Dell', 'Latitude']),
    ('Which display panel variant does this particular Poco F1 have?',
     ['README.md', 'current-system.md', 'hardware.md'], ['Tianma']),
    ('What IP addresses do the phone and computer use over the USB network?',
     ['README.md', 'current-system.md', 'usb-ssh.md', 'operations.md'],
     ['172.16.42.1', '172.16.42.2']),
    ('Why is the historical Kupfer boot image too large to flash directly?',
     ['build-patches.md', 'build-guide.md', 'flashing-guide.md', 'history.md'],
     ['ext2', 'aboot.img']),
]


def invoke(binary, operation, args=()):
    command = [str(binary), 'vector', operation, *args,
               '--project', PROJECT, '--root', str(WORKSPACE), '--store', str(STORE),
               '--timeout', '5m', '--json']
    result = subprocess.run(command, cwd=WORKSPACE, text=True,
                            stdout=subprocess.PIPE, check=True)
    return json.loads(result.stdout)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=Path.home() / 'llmcli')
    commands = parser.add_subparsers(dest='command', required=True)
    index = commands.add_parser('index')
    index.add_argument('--dry-run', action='store_true')
    index.add_argument('--provider', default='cloudflare')
    index.add_argument('--model', default='cloudflare/@cf/baai/bge-m3')
    commands.add_parser('update')
    commands.add_parser('status')
    commands.add_parser('files')
    search = commands.add_parser('search')
    search.add_argument('query')
    search.add_argument('--limit', type=int, default=5)
    commands.add_parser('test')
    args = parser.parse_args()
    if not args.binary.is_file():
        parser.error('llmcli binary not found: ' + str(args.binary))
    if args.command == 'index':
        options = [*SOURCES, '--provider', args.provider, '-m', args.model,
                   '--chunk-size', '1800', '--overlap', '180', '--batch-size', '16']
        if args.dry_run:
            options += ['--dry-run']
        result = invoke(args.binary, 'add', options)
    elif args.command == 'search':
        result = invoke(args.binary, 'search', [args.query, '--limit', str(args.limit)])
    elif args.command == 'test':
        results = []
        for query, paths, terms in CASES:
            response = invoke(args.binary, 'search', [query, '--limit', '8'])
            hits = response.get('results', [])
            matches = [rank for rank, hit in enumerate(hits, start=1)
                       if Path(hit.get('path', '')).name in paths
                       and all(term.lower() in hit.get('text', '').lower() for term in terms)]
            passed = bool(matches)
            results.append({'query': query, 'passed': passed,
                            'expected_source_names': paths, 'expected_terms': terms,
                            'matching_ranks': matches,
                            'response': response})
            print(('PASS' if passed else 'FAIL') + ': ' + query, flush=True)
        result = {'passed': sum(r['passed'] for r in results),
                  'total': len(results), 'checks': results}
        STORE.parent.mkdir(parents=True, exist_ok=True)
        report = STORE.parent / 'docs-vector-tests.json'
        report.write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
        print('Report:', report)
        print(f"{result['passed']}/{result['total']} retrieval checks passed")
        return 0 if result['passed'] == result['total'] else 1
    else:
        result = invoke(args.binary, args.command)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        sys.exit(str(error))
