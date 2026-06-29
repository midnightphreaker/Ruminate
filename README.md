# Ruminate

Rust Docker-first Streamable HTTP MCP server for session-local reflective workflow state.

Repository: `https://git.phrk.org/mcp-servers/ruminate`
Image: `git.phrk.org/mcp-servers/ruminate:latest`
Endpoint: `http://HOST:PORT/mcp`
Health: `http://HOST:PORT/healthz`

Ruminate stores all timeline, note, checkpoint, and gate data in memory per MCP session. It has no persistent user data.

## Quickstart

```bash
docker run --rm \
  --name ruminate \
  -p 8000:8000 \
  git.phrk.org/mcp-servers/ruminate:latest
```

Check health:

```bash
curl http://127.0.0.1:8000/healthz
```

Expected response:

```json
{"service":"ruminate","status":"ok"}
```

## Docker Smoke Test

```bash
docker build -t ruminate:test .
docker run --rm -p 8000:8000 ruminate:test
curl http://127.0.0.1:8000/healthz
```

The image runs as the non-root `app` user.

## Tools

| Tool | Purpose |
| --- | --- |
| `ruminate` | Records a reflective timeline entry. Compatible with the original `ruminate` output and also accepts optional `mode`, `tags`, `status`, and `confidence`. |
| `ruminate_note` | Records an `assumption`, `risk`, `decision`, `finding`, `question`, `blocker`, or `comparison`. |
| `ruminate_checkpoint` | Records a summary, open questions, next steps, status, and tags. |
| `ruminate_gate` | Records `plan`, `implementation`, `verification`, or `release` checks and returns `ready`. A gate is ready only when every check passes and has evidence. |
| `ruminate_inspect` | Reads `timeline`, `notes`, `checkpoints`, `gates`, or `summary` for the current session. |
| `ruminate_reflect` | Optional advisory LLM reflection. Disabled unless explicitly configured and never called implicitly. |

Original `ruminate` response shape:

```json
{
  "thoughtNumber": 1,
  "totalThoughts": 3,
  "nextThoughtNeeded": true,
  "branches": [],
  "thoughtHistoryLength": 1
}
```

## Resources

- `ruminate://session/timeline`
- `ruminate://session/notes`
- `ruminate://session/checkpoints`
- `ruminate://session/gates`

## Prompts

- `ruminate_plan`
- `ruminate_debug`
- `ruminate_review`
- `ruminate_verify`
- `ruminate_handoff`

## Optional Reflection

`ruminate_reflect` is disabled by default and always returns `advisory: true`. It does not persist prompts, completions, or credentials.

| Option | Default | Description |
| --- | --- | --- |
| `RUMINATE_LLM_ENABLED` | `false` | Enables advisory reflection only when set to `true`. |
| `RUMINATE_LLM_BASE_URL` | unset | OpenAI-compatible provider base URL. |
| `RUMINATE_LLM_API_KEY` | unset | Provider API key. |
| `RUMINATE_LLM_MODEL` | unset | Model name. |
| `RUMINATE_LLM_TIMEOUT_MS` | `30000` | Request timeout. |
| `RUMINATE_LLM_MAX_INPUT_CHARS` | `20000` | Maximum accepted input size. |

The release Docker image builds the default feature set. Build with the Rust `reflect` feature only when the optional provider client should be included.

## Runtime Options

| Option | Default | Description |
| --- | --- | --- |
| `PORT` | `8000` | Container listen port. Publish it with `-p HOST_PORT:PORT`. |
| `/mcp` | n/a | Streamable HTTP MCP endpoint. |
| `/healthz` | n/a | Health endpoint returning service status. |

## MCP Inspector Checklist

1. Start the server with `cargo run -p ruminate` or Docker.
2. Connect MCP Inspector to `http://127.0.0.1:8000/mcp`.
3. List tools and confirm all six Ruminate tools are present.
4. Call `ruminate` and confirm `thoughtHistoryLength` increments.
5. Add a note, checkpoint, and gate, then inspect `summary`.
6. List resources and read each `ruminate://session/*` URI.
7. List prompts and fetch each static prompt.
8. Open a second client session and confirm its summary starts empty.

Evaluation scenarios live in `evals/ruminate_scenarios.md`.

## Development

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --features reflect
docker build -t ruminate:test .
```

Run locally:

```bash
cargo run -p ruminate
```

## Forgejo Build

This repository includes `.forgejo/workflows/docker.yml` and `VERSION`.

On each push, the workflow builds and pushes `git.phrk.org/mcp-servers/ruminate:<VERSION>` and `:latest`, then increments `VERSION` by `0.0.1` after a successful push.

Forgejo registry secrets `REGISTRY_USER` and `REGISTRY_PASSWORD` are optional when the Forgejo-provided `GITHUB_TOKEN` can publish packages. The runner uses `git.phrk.org/mcp-servers/runner-image-docker-cli:latest`, which includes the Docker CLI.

