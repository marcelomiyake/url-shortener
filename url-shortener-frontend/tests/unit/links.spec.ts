import { afterEach, describe, expect, it, vi } from 'vitest';

import { createShortLink, LinkApiError } from '../../src/services/links';

describe('createShortLink', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('posts the destination and returns a valid relative short path', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ code: '0123456789AB', short_path: '/0123456789AB' }), {
        status: 201,
        headers: { 'content-type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(createShortLink('https://example.com/article')).resolves.toEqual({
      code: '0123456789AB',
      short_path: '/0123456789AB',
    });
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/links', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ destination_url: 'https://example.com/article' }),
    });
  });

  it('surfaces API validation errors without changing their meaning', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        error: { code: 'invalid_destination_url', message: 'Use an HTTP(S) URL.' },
      }), { status: 422, headers: { 'content-type': 'application/json' } }),
    ));

    await expect(createShortLink('javascript:alert(1)')).rejects.toMatchObject({
      name: 'LinkApiError',
      code: 'invalid_destination_url',
      message: 'Use an HTTP(S) URL.',
    });
  });

  it('rejects a server response that could create an off-origin link', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ code: '0123456789AB', short_path: '//evil.example/' }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }),
    ));

    await expect(createShortLink('https://example.com/')).rejects.toBeInstanceOf(LinkApiError);
  });

  it('reports network failures with a user-facing message', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('offline')));

    await expect(createShortLink('https://example.com/')).rejects.toMatchObject({
      code: 'network_error',
      message: 'The service could not be reached. Try again later.',
    });
  });

  it('passes cancellation to fetch and reports that the write may have completed', async () => {
    const controller = new AbortController();
    controller.abort();
    const fetchMock = vi.fn().mockRejectedValue(new DOMException('Aborted', 'AbortError'));
    vi.stubGlobal('fetch', fetchMock);

    await expect(createShortLink('https://example.com/', controller.signal)).rejects.toMatchObject({
      code: 'request_cancelled',
      message: expect.stringContaining('The link may already exist'),
    });
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/links', expect.objectContaining({ signal: controller.signal }));
  });
});
