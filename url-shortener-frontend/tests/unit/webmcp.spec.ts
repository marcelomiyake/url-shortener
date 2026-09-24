import { afterEach, describe, expect, it, vi } from 'vitest';
import type { WebMCP } from 'webmcp-types';

import { registerShortLinkTool } from '../../src/services/webmcp';

describe('WebMCP short-link tool', () => {
  afterEach(() => vi.restoreAllMocks());

  it('does not register tools when the browser does not implement WebMCP', () => {
    const createLink = vi.fn();

    expect(registerShortLinkTool(undefined, createLink, 'https://short.example')).toBeNull();
    expect(registerShortLinkTool({} as WebMCP.ModelContext, createLink, 'https://short.example')).toBeNull();
    expect(createLink).not.toHaveBeenCalled();
  });

  it('registers a consequential same-origin tool and returns the created Short URL', async () => {
    const tools: WebMCP.ModelContextTool[] = [];
    let registrationSignal: AbortSignal | undefined;
    const context = {
      registerTool: vi.fn((tool: WebMCP.ModelContextTool, options?: WebMCP.ModelContextRegisterToolOptions) => {
        tools.push(tool);
        registrationSignal = options?.signal;
        return Promise.resolve();
      }),
    } as unknown as WebMCP.ModelContext;
    const createLink = vi.fn().mockResolvedValue({ code: '0123456789AB', short_path: '/0123456789AB' });
    const requestController = new AbortController();
    const registration = registerShortLinkTool(context, createLink, 'https://short.example');

    await registration?.ready;
    expect(context.registerTool).toHaveBeenCalledOnce();
    expect(tools[0]?.name).toBe('create-short-link');
    expect(tools[0]?.annotations).toMatchObject({ readOnlyHint: false, consequentialHint: true });
    expect(registrationSignal?.aborted).toBe(false);

    const execute = tools[0]?.execute as WebMCP.ToolExecuteCallback<{ destinationUrl: string }>;
    const result = await execute({ destinationUrl: 'https://example.com/article' }, { signal: requestController.signal });

    expect(createLink).toHaveBeenCalledWith('https://example.com/article', requestController.signal);
    expect(result).toEqual({
      content: [{ type: 'text', text: 'Short link ready: https://short.example/0123456789AB' }],
    });
    expect(JSON.stringify(result)).not.toContain('https://example.com/article');

    registration?.unregister();
    expect(registrationSignal?.aborted).toBe(true);
  });

  it('rejects missing destinations before calling the API', async () => {
    const tools: WebMCP.ModelContextTool[] = [];
    const context = {
      registerTool: vi.fn((tool: WebMCP.ModelContextTool) => {
        tools.push(tool);
        return Promise.resolve();
      }),
    } as unknown as WebMCP.ModelContext;
    const createLink = vi.fn();
    const registration = registerShortLinkTool(context, createLink, 'https://short.example');
    await registration?.ready;

    const execute = tools[0]?.execute as WebMCP.ToolExecuteCallback<{ destinationUrl: string }>;
    await expect(execute({ destinationUrl: ' ' }, { signal: new AbortController().signal }))
      .rejects.toThrow('Provide an HTTP or HTTPS destination address.');
    expect(createLink).not.toHaveBeenCalled();
    registration?.unregister();
  });

  it('treats a rejected browser registration as optional enhancement failure', async () => {
    let registrationSignal: AbortSignal | undefined;
    const context = {
      registerTool: vi.fn((_tool: WebMCP.ModelContextTool, options?: WebMCP.ModelContextRegisterToolOptions) => {
        registrationSignal = options?.signal;
        return Promise.reject(new DOMException('Permission denied.', 'NotAllowedError'));
      }),
    } as unknown as WebMCP.ModelContext;
    const registration = registerShortLinkTool(context, vi.fn(), 'https://short.example');

    await expect(registration?.ready).resolves.toBeUndefined();
    expect(registrationSignal?.aborted).toBe(true);
  });
});
