import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

describe('Forgejo Docker workflow', () => {
  it('publishes the ruminate image described in the README', () => {
    const workflow = readFileSync('.forgejo/workflows/docker.yml', 'utf8');

    expect(workflow).toContain('git.phrk.org/mcp-servers/ruminate:${{ steps.version.outputs.version }}');
    expect(workflow).toContain('git.phrk.org/mcp-servers/ruminate:latest');
    expect(workflow).toContain('org.opencontainers.image.source=https://git.phrk.org/mcp-servers/ruminate');
    expect(workflow).not.toContain('mcp-servers/sequentialthinking');
  });
});
