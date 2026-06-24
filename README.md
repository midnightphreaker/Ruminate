# Sequential Thinking MCP Server

Docker-only Streamable HTTP deployment for the Sequential Thinking MCP server.

Repository: `https://git.phrk.org/mcp-servers/sequentialthinking`
Image: `git.phrk.org/mcp-servers/sequentialthinking:latest`
Endpoint: `http://HOST:PORT/mcp`

## Quickstart

```bash
docker run --rm \
  --name sequentialthinking-mcp \
  -p 8000:8000 \
  git.phrk.org/mcp-servers/sequentialthinking:latest
```

There is no persistent user data for this service.

## Runtime Options

| Option | Default | Description |
| --- | --- | --- |
| `PORT` | `8000` | Container listen port. Publish it with `-p HOST_PORT:PORT`. |
| `DISABLE_THOUGHT_LOGGING` | `true` | Set to `false` to allow thought logging to container logs. |
| `/mcp` | n/a | Streamable HTTP MCP endpoint. |

Example with thought logging enabled:

```bash
docker run --rm \
  --name sequentialthinking-mcp \
  -e DISABLE_THOUGHT_LOGGING=false \
  -p 8000:8000 \
  git.phrk.org/mcp-servers/sequentialthinking:latest
```

## Forgejo Build

This repository includes `.forgejo/workflows/docker.yml` and `VERSION`.

On each push, the workflow builds and pushes `git.phrk.org/mcp-servers/sequentialthinking:<VERSION>` and `:latest`, then increments `VERSION` by `0.0.1` after a successful push.

Forgejo registry secrets `REGISTRY_USER` and `REGISTRY_PASSWORD` are optional when the Forgejo-provided `GITHUB_TOKEN` can publish packages. The runner uses `git.phrk.org/mcp-servers/runner-image-docker-cli:latest`, which includes the Docker CLI.
