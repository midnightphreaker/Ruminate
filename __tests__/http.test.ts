import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StreamableHTTPClientTransport } from '@modelcontextprotocol/sdk/client/streamableHttp.js';
import type express from 'express';
import { once } from 'node:events';
import type { AddressInfo } from 'node:net';
import type { Server } from 'node:http';
import { afterEach, describe, expect, it } from 'vitest';

type AppModule = {
  createApp?: () => express.Express;
};

let servers: Server[] = [];

async function startApp() {
  process.env.PORT = '0';
  const mod = await import('../index.js') as AppModule;
  expect(typeof mod.createApp).toBe('function');

  const app = mod.createApp!();
  const server = app.listen(0, '127.0.0.1');
  servers.push(server);
  await once(server, 'listening');

  const { port } = server.address() as AddressInfo;
  return {
    baseUrl: new URL(`http://127.0.0.1:${port}`),
    mcpUrl: new URL(`http://127.0.0.1:${port}/mcp`),
  };
}

async function createClient(url: URL) {
  const client = new Client({ name: 'ruminate-test-client', version: '1.0.0' });
  const transport = new StreamableHTTPClientTransport(url);
  await client.connect(transport);
  return { client, transport };
}

async function callRuminate(client: Client, thought: string) {
  return client.callTool({
    name: 'ruminate',
    arguments: {
      thought,
      thoughtNumber: 1,
      totalThoughts: 3,
      nextThoughtNeeded: true,
    },
  });
}

afterEach(async () => {
  await Promise.all(servers.map((server) => new Promise<void>((resolve, reject) => {
    server.close((error) => error ? reject(error) : resolve());
  })));
  servers = [];
});

describe('HTTP app', () => {
  it('serves health status without requiring an MCP session', async () => {
    const { baseUrl } = await startApp();

    const response = await fetch(new URL('/healthz', baseUrl));

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual({
      status: 'ok',
      service: 'ruminate',
    });
  });

  it('keeps thought history isolated per MCP HTTP session', async () => {
    const { mcpUrl } = await startApp();
    const first = await createClient(mcpUrl);
    const second = await createClient(mcpUrl);

    try {
      const firstResult = await callRuminate(first.client, 'first session thought');
      const secondResult = await callRuminate(second.client, 'second session thought');

      expect(firstResult.structuredContent).toMatchObject({ thoughtHistoryLength: 1 });
      expect(secondResult.structuredContent).toMatchObject({ thoughtHistoryLength: 1 });
      expect(first.transport.sessionId).toBeTruthy();
      expect(second.transport.sessionId).toBeTruthy();
      expect(first.transport.sessionId).not.toBe(second.transport.sessionId);
    } finally {
      await first.client.close();
      await second.client.close();
    }
  });
});
