# Ruminate

Ruminate is a Rust Streamable HTTP MCP server for session-local reflective workflow state. It gives MCP clients tools, prompts, and resources for recording thoughts, notes, checkpoints, verification gates, and optional advisory reflection during an agent session.

The server stores workflow data in memory through the MCP session manager. It does not persist timeline entries, notes, checkpoints, gates, prompts, completions, or credentials.

Repository: `https://git.phrk.org/pub/Ruminate`
Docker image: `git.phrk.org/pub/ruminate:latest`

## Quickstart

Run the released container:

```bash
docker run --rm \
  --name ruminate \
  -p 8000:8000 \
  git.phrk.org/pub/ruminate:latest
```

Check the health endpoint:

```bash
curl http://127.0.0.1:8000/healthz
```

Expected response:

```json
{"service":"ruminate","status":"ok"}
```

Connect an MCP client or MCP Inspector to:

```text
http://127.0.0.1:8000/mcp
```

For local source development, run the crate directly:

```bash
cargo run -p ruminate
```

## Stdio

Select stdio explicitly when an MCP client launches the server as a local process. It uses stdin and stdout for MCP messages and opens no HTTP port:

```bash
cargo build --release -p ruminate
RUMINATE_TRANSPORT=stdio ./target/release/ruminate
```

Register the absolute release binary with Codex:

```bash
codex mcp add ruminate \
  --env RUMINATE_TRANSPORT=stdio \
  -- /absolute/path/to/Ruminate/target/release/ruminate
```

For a containerized stdio server, keep the container interactive and set the same transport selector:

```bash
docker run --rm -i \
  -e RUMINATE_TRANSPORT=stdio \
  git.phrk.org/pub/ruminate:latest
```

Stdio state is session-local and in memory: records are available only while that launched server process remains connected, and are discarded when it exits. Leave `RUMINATE_TRANSPORT` unset to retain the Streamable HTTP/Docker default.

## What The Server Provides

Ruminate exposes one Streamable HTTP MCP endpoint at `/mcp` and one HTTP health endpoint at `/healthz`. Each MCP session gets its own in-memory `RuminateState`, so records created by one client session are isolated from another client session.

The MCP server advertises tools, prompts, and resources.

### Tools

| Tool | Purpose |
| --- | --- |
| `ruminate` | Record a reflective timeline entry. It accepts the original `ruminate` thought fields and optional metadata such as `mode`, `tags`, `status`, and `confidence`. |
| `ruminate_note` | Record an `assumption`, `risk`, `decision`, `finding`, `question`, `blocker`, or `comparison`. |
| `ruminate_checkpoint` | Record a workflow summary with open questions, next steps, status, and tags. |
| `ruminate_gate` | Record a `plan`, `implementation`, `verification`, or `release` gate. A gate is ready only when it has checks and every check passes with non-empty evidence. |
| `ruminate_inspect` | Read the current session's `timeline`, `notes`, `checkpoints`, `gates`, or compact `summary`. |
| `ruminate_reflect` | Request optional advisory LLM reflection. It is disabled by default and is never called implicitly. |

The original `ruminate` response shape is preserved:

```json
{
  "thoughtNumber": 1,
  "totalThoughts": 3,
  "nextThoughtNeeded": true,
  "branches": [],
  "thoughtHistoryLength": 1
}
```

### Prompts

| Prompt | Purpose |
| --- | --- |
| `ruminate_plan` | Capture assumptions, risks, checkpoints, and a plan gate before implementation. |
| `ruminate_debug` | Track hypotheses, findings, blockers, and verification evidence while debugging. |
| `ruminate_review` | Record review findings, risks, decisions, and follow-up questions. |
| `ruminate_verify` | Record checks, statuses, and evidence before claiming completion. |
| `ruminate_handoff` | Prepare a handoff from summaries, checkpoints, open questions, and gates. |

### Resources

| Resource URI | Contents |
| --- | --- |
| `ruminate://session/timeline` | JSON timeline entries for the current session. |
| `ruminate://session/notes` | JSON notes for the current session. |
| `ruminate://session/checkpoints` | JSON checkpoints for the current session. |
| `ruminate://session/gates` | JSON gates for the current session. |

## Configuration

### Runtime

| Variable | Default | Description |
| --- | --- | --- |
| `PORT` | `8000` | HTTP listen port used by the binary and container. |
| `RUMINATE_ALLOWED_HOSTS` | Streamable HTTP server defaults | Comma-separated hostnames or `host:port` authorities added to the Streamable HTTP Host guard. Set this for public Docker deployments. |
| `RUST_LOG` | `ruminate=info,tower_http=info` | Tracing filter used by `tracing-subscriber` when no filter is already set. |

The container exposes port `8000` and runs the compiled binary as a non-root `app` user.

### Optional Reflection

`ruminate_reflect` always returns `advisory: true`. Without the `reflect` Cargo feature, enabling reflection reports `feature_disabled`. With the `reflect` feature, the server can call an OpenAI-compatible `/chat/completions` endpoint when all required provider settings are present.

| Variable | Default | Description |
| --- | --- | --- |
| `RUMINATE_LLM_ENABLED` | `false` | Enables advisory reflection when set to `true`. |
| `RUMINATE_LLM_BASE_URL` | unset | OpenAI-compatible provider base URL. |
| `RUMINATE_LLM_API_KEY` | unset | Provider API key. |
| `RUMINATE_LLM_MODEL` | unset | Model name. |
| `RUMINATE_LLM_TIMEOUT_MS` | `30000` | Provider request timeout in milliseconds. |
| `RUMINATE_LLM_MAX_INPUT_CHARS` | `20000` | Maximum accepted reflection input size. |

Build or test the reflection client with:

```bash
cargo test --workspace --features reflect
```

## Local Development

This is a Cargo workspace with one member crate at `crates/ruminate`.

Run the server:

```bash
cargo run -p ruminate
```

Run the usual checks:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Build and smoke-test the Docker image:

```bash
docker build -t ruminate:test .
docker run --rm -p 8000:8000 ruminate:test
curl http://127.0.0.1:8000/healthz
```

Evaluation scenarios for MCP Inspector or another MCP client live in `evals/ruminate_scenarios.md`.

## MCP Inspector Checklist

1. Start the server with `cargo run -p ruminate` or Docker.
2. Connect MCP Inspector to `http://127.0.0.1:8000/mcp`.
3. List tools and confirm the six Ruminate tools are present.
4. Call `ruminate` and confirm `thoughtHistoryLength` increments.
5. Add a note, checkpoint, and gate, then inspect `summary`.
6. List resources and read each `ruminate://session/*` URI.
7. List prompts and fetch each static prompt.
8. Open a second client session and confirm its summary starts empty.

## Automated Forgejo releases

The [Release action](https://git.phrk.org/pub/Ruminate/actions) checks and builds Linux AMD64 and Windows x86_64 archives plus a Linux container. It runs on the `linux-amd64` Forgejo runners. Archive names are `Ruminate.vVERSION-linux-amd64.tar.gz` (and `Ruminate.vVERSION-windows-x86_64.zip` when supported).

Run the workflow manually with **Major** to increment the middle version component and reset the last (`0.1.7` → `0.2.0`), **minor** to increment the last (`0.1.7` → `0.1.8`), or **retry** to publish the current version without another bump. Use **retry** for the initial release or to resume a failed publication. Rerunning the original failed job also retains its version.

A default-branch push releases only when an application version changes. Change `VERSION`; the workflow synchronizes the configured application manifests and lockfile entries. Ordinary source/documentation pushes do not release. Checks and all builds finish before the workflow commits version changes. Release notes include up to ten commit subjects since the preceding release.

Downloads appear on the [releases page](https://git.phrk.org/pub/Ruminate/releases). Container tags are `git.phrk.org/pub/ruminate:vVERSION` and `git.phrk.org/pub/ruminate:latest`; `latest` changes only after the required release artifacts have uploaded. Container repository names use lowercase as required by the registry.

The repository requires Actions secrets `RELEASE_TOKEN` (repository/release/package write access) and `RELEASE_USER` (its account name). Keep values in the secret store. The runner's existing Podman socket must be available at `/var/run/docker.sock`; the workflow uses its remote API. Build commands, version fields, and required output filenames are declared in `.forgejo/release.json`.
