# Ruminate

Docker-first Rust Streamable HTTP MCP server for session-local reflective workflow state.

Repository: `https://git.phrk.org/mcp-servers/ruminate`
Image: `git.phrk.org/mcp-servers/ruminate:latest`
Endpoint: `http://HOST:PORT/mcp`

## Quickstart

```bash
docker run --rm \
  --name ruminate \
  -p 8000:8000 \
  git.phrk.org/mcp-servers/ruminate:latest
```

There is no persistent user data for this service. Timeline, notes, checkpoints, and gates live only in the current MCP session.

## Docker Smoke Test

```bash
docker build -t ruminate:test .
docker run --rm -p 8000:8000 ruminate:test
curl http://127.0.0.1:8000/healthz
```

## Runtime Options

| Option | Default | Description |
| --- | --- | --- |
| `PORT` | `8000` | Container listen port. Publish it with `-p HOST_PORT:PORT`. |
| `DISABLE_THOUGHT_LOGGING` | `true` | Set to `false` to allow thought logging to container logs. |
| `RUMINATE_LLM_ENABLED` | `false` | Enables advisory `ruminate_reflect` only when explicitly set to `true` and the other LLM variables are configured. |
| `RUMINATE_LLM_BASE_URL` | unset | OpenAI-compatible base URL for optional reflection. |
| `RUMINATE_LLM_API_KEY` | unset | API key for optional reflection. |
| `RUMINATE_LLM_MODEL` | unset | Model name for optional reflection. |
| `RUMINATE_LLM_TIMEOUT_MS` | `30000` | Timeout for optional reflection requests. |
| `RUMINATE_LLM_MAX_INPUT_CHARS` | `20000` | Maximum input size accepted by `ruminate_reflect`. |
| `/mcp` | n/a | Streamable HTTP MCP endpoint. |

## Tools

- `ruminate`: session-local timeline entry compatible with the original tool. Also accepts optional `mode`, `tags`, `status`, and `confidence`.
- `ruminate_note`: records an `assumption`, `risk`, `decision`, `finding`, `question`, `blocker`, or `comparison`.
- `ruminate_checkpoint`: records a summary, open questions, next steps, status, and tags.
- `ruminate_gate`: records `plan`, `implementation`, `verification`, or `release` checks and returns `ready`.
- `ruminate_inspect`: reads `timeline`, `notes`, `checkpoints`, `gates`, or `summary`.
- `ruminate_reflect`: optional advisory LLM reflection. Disabled unless configured and never called implicitly.

## Resources And Prompts

Resources:

- `ruminate://session/timeline`
- `ruminate://session/notes`
- `ruminate://session/checkpoints`
- `ruminate://session/gates`

Prompts:

- `ruminate_plan`
- `ruminate_debug`
- `ruminate_review`
- `ruminate_verify`
- `ruminate_handoff`

## MCP Inspector Checklist

1. Start the server with `cargo run -p ruminate` or Docker.
2. Connect MCP Inspector to `http://127.0.0.1:8000/mcp`.
3. List tools and confirm the six Ruminate tools are present.
4. Call `ruminate` and confirm `thoughtHistoryLength` increments.
5. Add a note, checkpoint, and gate, then inspect `summary`.
6. List resources and read each `ruminate://session/*` URI.
7. List prompts and fetch each static prompt.
8. Open a second client session and confirm its summary starts empty.

Evaluation scenarios live in `evals/ruminate_scenarios.md`.

Example with thought logging enabled:

```bash
docker run --rm \
  --name ruminate \
  -e DISABLE_THOUGHT_LOGGING=false \
  -p 8000:8000 \
  git.phrk.org/mcp-servers/ruminate:latest
```

## Forgejo Build

This repository includes `.forgejo/workflows/docker.yml` and `VERSION`.

On each push, the workflow builds and pushes `git.phrk.org/mcp-servers/ruminate:<VERSION>` and `:latest`, then increments `VERSION` by `0.0.1` after a successful push.

Forgejo registry secrets `REGISTRY_USER` and `REGISTRY_PASSWORD` are optional when the Forgejo-provided `GITHUB_TOKEN` can publish packages. The runner uses `git.phrk.org/mcp-servers/runner-image-docker-cli:latest`, which includes the Docker CLI.
