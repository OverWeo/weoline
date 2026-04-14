# weoline

Fast, cross-platform Claude Code statusline written in Rust. Displays context window usage, token counts, cache statistics, session totals, and API rate limits as ANSI-colored output.

## Features

- Context window usage with colored progress bar
- Current token counts (input/output)
- Cache statistics (read/write)
- Session totals
- API rate limits (5-hour, 7-day, Sonnet) with countdown timers
- Background refresh with file-locked coordination
- Three display modes: full, compact, minimal
- Cross-platform: macOS, Linux, Windows
- Platform-optimized TLS: native-tls on macOS/Windows, rustls on Linux

## Installation

### NPM (Zero-Overhead Hoisted Binary)

For users in JS/TS ecosystems. Executes the native binary directly — no Node runtime overhead.

```bash
npx weoline
# OR
pnpm add -D weoline
```

### Cargo

```bash
cargo install weoline
```

### Homebrew (macOS / Linux)

```bash
brew install overweo/tap/weoline
```

### Direct Download (Shell Script)

For CI environments or users without package managers.

```bash
curl -fsSL https://raw.githubusercontent.com/OverWeo/weoline/main/misc/install.sh | bash
```

### Build from Source

```bash
git clone https://github.com/OverWeo/weoline.git
cd weoline
cargo build --release
cp target/release/weoline ~/.local/bin/
```

## Quick Start

Add to your Claude Code settings (`~/.claude/settings.json`):

```json
{
  "env": {
    "SL_MODE": "full"
  },
  "statusLine": {
    "type": "command",
    "command": "weoline"
  }
}
```

## Build Variants

| Variant | Command | Includes | Binary Size |
|---------|---------|----------|-------------|
| **Full** (default) | `cargo build --release` | Context + tokens + cache + session + API rate limits + background fetch + query mode | ~789 KB (macOS) / ~1.3 MB (Linux) |
| **Minimal** | `cargo build --release --no-default-features` | Context + tokens + cache + session only (from stdin) | ~345 KB |

**Full** uses native-tls on macOS/Windows (OS TLS stack) and rustls on Linux (no OpenSSL dependency).

**Minimal** disables all network dependencies. Reads stdin JSON from Claude Code and renders context/token/session info only.

## Configuration

All settings are via `SL_*` environment variables, configurable in Claude Code's `settings.json` under `"env"`:

| Variable | Default | Description |
|----------|---------|-------------|
| `SL_MODE` | `full` | Display mode: `full`, `compact`, `minimal` |
| `SL_SHOW_TOKENS` | `0` | Show current token counts (`1` to enable) |
| `SL_SHOW_CACHE` | `1` | Show cache read/write stats (`0` to disable) |
| `SL_SHOW_SESSION` | `1` | Show session totals (`0` to disable) |
| `SL_SHOW_LIMITS` | `1` | Show API rate limits (`0` to disable) |
| `SL_SHOW_WEEKLY` | `1` | Show 7-day limit (`0` to disable) |
| `SL_SHOW_SONNET` | `1` | Show Sonnet limit (`0` to disable) |
| `SL_REFRESH_INTERVAL` | `300` | Cache refresh interval in seconds |
| `SL_BAR_WIDTH` | `14` | Progress bar width in characters |
| `SL_CREDENTIALS_FILE` | `~/.claude/.credentials.json` | Path to OAuth credentials file |
| `SL_CACHE_FILE` | `~/.claude/usage-cache.json` | Path to rate limit cache file |

## Display Modes

**Full** — context | tokens + cache | session | limits:
```
🧠 ██████░░░░░░░░ 45% (90k/200k)  |  💾 cache read: 10k  write: 3k  |  🔄 session: 📥 in: 50k  📤 out: 20k  |  ⏱ 5h: 24% ↻3h52m  📅 7d: 8%  🎵 sonnet: 5%
```

**Compact** — context | limits:
```
🧠 ██████░░░░░░░░ 45% (90k/200k)  |  ⏱ 5h: 24% ↻3h52m  📅 7d: 8%  🎵 sonnet: 5%
```

**Minimal** — context only:
```
🧠 ██████░░░░░░░░ 45% (90k/200k)
```

## Query Mode

Query cached rate limit data programmatically — no stdin required.

```bash
weoline --query                              # toon, full detail, all buckets
weoline --query --format json                # JSON, full detail, all buckets
weoline --query -f json -d minimal           # JSON, just percentages
weoline --query -f json --filter sonnet      # JSON, sonnet bucket only
weoline --query --refresh --format json      # force fresh API fetch, then JSON
```

### Query Flags

| Flag | Short | Values | Default | Description |
|------|-------|--------|---------|-------------|
| `--query` | `-q` | (presence) | off | Activate query mode |
| `--format` | `-f` | `json`, `toon` | `toon` | Output format |
| `--detail` | `-d` | `minimal`, `full` | `full` | Detail level |
| `--filter` | | `all`, `sonnet`, `five-hour`, `seven-day` | `all` | Filter to specific bucket |
| `--refresh` | `-r` | (presence) | off | Force fresh API fetch before query (blocking) |
| `--version` | `-V` | (presence) | off | Print version |

### JSON Output

**Full detail** (`--format json`):
```json
{
  "five_hour": {
    "utilization": 24.0,
    "resets_at": "2026-04-06T18:00:00+00:00",
    "resets_in": "3h52m"
  },
  "seven_day": {
    "utilization": 8.0,
    "resets_at": "2026-04-10T00:00:00+00:00",
    "resets_in": "3d0h"
  },
  "seven_day_sonnet": {
    "utilization": 5.0,
    "resets_at": "2026-04-10T00:00:00+00:00",
    "resets_in": "3d0h"
  },
  "meta": {
    "fetched_at": 1743955200,
    "is_stale": false
  }
}
```

**Minimal detail** (`-f json -d minimal`):
```json
{
  "five_hour_pct": 24.0,
  "seven_day_pct": 8.0,
  "seven_day_sonnet_pct": 5.0,
  "is_stale": false
}
```

> **Note:** `--refresh` performs a blocking HTTP request. Use it for explicit user invocations, not automated statusline hooks.

## How It Works

```
Claude Code → stdin (JSON) → weoline → stdout (ANSI)
                                ↓
                    Background: --fetch → API → cache file

weoline --query → read cache → stdout (JSON/toon)
         ↓ (--refresh)
       API → cache file → read → stdout
```

1. Claude Code pipes context window JSON to stdin
2. `weoline` parses JSON, reads cached rate limits, renders ANSI output to stdout
3. If the cache is stale, spawns a detached `weoline --fetch` subprocess
4. The fetch subprocess acquires a file lock (OS-native via `fd-lock`), calls the Anthropic usage API, writes the cache atomically, and exits
5. File lock is auto-released by the OS on process exit or crash
6. Query mode (`--query`) reads the cache directly and outputs structured data (no stdin needed)

## Cross-Platform Notes

- **macOS**: Uses native-tls (Security.framework). OAuth token read from Keychain first, then credentials file. Keychain access may prompt for password/Touch ID.
- **Linux**: Uses rustls (no OpenSSL dependency). OAuth token from credentials file only.
- **Windows**: Uses native-tls (SChannel). ANSI escape codes enabled via `SetConsoleMode`. `CREATE_NO_WINDOW` flag prevents console flash on background fetch spawn.

## Testing

```bash
# Unit tests
cargo test

# Smoke tests (builds release, tests pipe/query/help/error modes)
./tests/smoke-test.sh

# Manual pipe tests
echo '{"context_window":{"used_percentage":45,"context_window_size":200000}}' | ./target/release/weoline

# Empty input (graceful)
echo '{}' | ./target/release/weoline

# Mode tests
echo '{"context_window":{"used_percentage":45,"context_window_size":200000}}' | SL_MODE=compact ./target/release/weoline
echo '{"context_window":{"used_percentage":45,"context_window_size":200000}}' | SL_MODE=minimal ./target/release/weoline
```

## License

Apache-2.0
