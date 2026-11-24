#!/usr/bin/env python3
"""Retrieve and display the public (WAN) IP address."""

import ipaddress
import json
import os
import sys
import urllib.parse
import urllib.request

VERSION = "1.1.0"
TIMEOUT = 10
SOURCES = (
    "https://api.ipify.org?format=json",
    "https://api64.ipify.org?format=json",
)

RESET = "\x1b[0m"
BOLD = "\x1b[1m"
DIM = "\x1b[2m"
CYAN = "\x1b[36m"
GREEN = "\x1b[32m"
RED = "\x1b[31m"


def fetch_ip(url):
    request = urllib.request.Request(
        url, headers={"User-Agent": f"wan-ip/{VERSION}"}
    )
    with urllib.request.urlopen(request, timeout=TIMEOUT) as response:
        body = response.read().decode("utf-8").strip()
    if url.endswith("format=json"):
        return json.loads(body)["ip"]
    return body


def use_color():
    return (
        sys.stdout.isatty()
        and "NO_COLOR" not in os.environ
        and os.environ.get("TERM", "") != "dumb"
    )


def display(ip, source):
    try:
        label = f"WAN IPv{ipaddress.ip_address(ip).version}"
    except ValueError:
        label = "WAN IP"
    detail = f"via {source}"

    width = max(len(label), len(ip), len(detail))
    color = use_color()

    def paint(code, text):
        return f"{code}{text}{RESET}" if color else text

    def row(text, style=""):
        padding = " " * (width - len(text))
        content = paint(style, text) if style else text
        return f"  {paint(CYAN, '│')}  {content}{padding}  {paint(CYAN, '│')}"

    edge = paint(CYAN, f"╭{'─' * (width + 4)}╮")
    bottom = paint(CYAN, f"╰{'─' * (width + 4)}╯")

    print()
    print(f"  {edge}")
    print(row(label, BOLD))
    print(row(ip, GREEN))
    print(row(detail, DIM))
    print(f"  {bottom}")
    print()


def main():
    errors = []
    for url in SOURCES:
        try:
            ip = fetch_ip(url)
            if ip:
                host = urllib.parse.urlsplit(url).hostname
                display(ip, host)
                return 0
        except Exception as exc:
            errors.append(f"{url}: {exc}")

    color = use_color()
    mark = f"{RED}✖{RESET}" if color else "✖"
    print(f"{mark} Could not retrieve WAN IP address.", file=sys.stderr)
    for error in errors:
        print(f"  {error}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
