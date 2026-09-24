import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { WebMCP } from 'webmcp-types';

import App from '../../src/App.vue';

describe('Short Form', () => {
  let clipboardDescriptor: PropertyDescriptor | undefined;

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
    if (clipboardDescriptor) {
      Object.defineProperty(navigator, 'clipboard', clipboardDescriptor);
    } else {
      Reflect.deleteProperty(navigator, 'clipboard');
    }
    Reflect.deleteProperty(document, 'modelContext');
  });

  it('shows the created same-origin link and copy confirmation', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    clipboardDescriptor = Object.getOwnPropertyDescriptor(navigator, 'clipboard');
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText },
    });
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ code: '0123456789AB', short_path: '/0123456789AB' }), {
        status: 201,
        headers: { 'content-type': 'application/json' },
      }),
    ));
    const wrapper = mount(App);

    await wrapper.get('#destination-url').setValue('https://example.com/article');
    await wrapper.get('form').trigger('submit');
    await flushPromises();

    const link = wrapper.get('[data-testid="short-link"]');
    expect(link.text()).toBe(`${window.location.origin}/0123456789AB`);
    expect(link.attributes('href')).toBe(`${window.location.origin}/0123456789AB`);

    await wrapper.get('button.button-secondary').trigger('click');
    await flushPromises();
    expect(writeText).toHaveBeenCalledWith(`${window.location.origin}/0123456789AB`);
    expect(wrapper.get('output').text()).toBe('Short link copied.');
    wrapper.unmount();
  });

  it('announces a validation error and marks the URL field invalid', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        error: { code: 'invalid_destination_url', message: 'Destination URL must use HTTP or HTTPS.' },
      }), { status: 422, headers: { 'content-type': 'application/json' } }),
    ));
    const wrapper = mount(App);

    await wrapper.get('#destination-url').setValue('ftp://example.com/file');
    await wrapper.get('form').trigger('submit');
    await flushPromises();

    expect(wrapper.get('#destination-url').attributes('aria-invalid')).toBe('true');
    expect(wrapper.get('[role="alert"]').text()).toBe('Destination URL must use HTTP or HTTPS.');
    wrapper.unmount();
  });

  it('announces service failures without reflecting server internals', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('database password leaked by a test double')));
    const wrapper = mount(App);

    await wrapper.get('#destination-url').setValue('https://example.com/');
    await wrapper.get('form').trigger('submit');
    await flushPromises();

    expect(wrapper.get('[role="alert"]').text()).toBe('The service could not be reached. Try again later.');
    expect(wrapper.text()).not.toContain('database password');
    wrapper.unmount();
  });

  it('creates links through WebMCP and updates the visible result panel', async () => {
    let registeredTool: WebMCP.ModelContextTool | undefined;
    let registrationSignal: AbortSignal | undefined;
    const context = {
      registerTool: vi.fn((tool: WebMCP.ModelContextTool, options?: WebMCP.ModelContextRegisterToolOptions) => {
        registeredTool = tool;
        registrationSignal = options?.signal;
        return Promise.resolve();
      }),
    } as unknown as WebMCP.ModelContext;
    Object.defineProperty(document, 'modelContext', {
      configurable: true,
      value: context,
    });
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ code: '0123456789AB', short_path: '/0123456789AB' }), {
        status: 201,
        headers: { 'content-type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);
    const wrapper = mount(App);
    await flushPromises();

    const execute = registeredTool?.execute as WebMCP.ToolExecuteCallback<{ destinationUrl: string }>;
    const result = await execute(
      { destinationUrl: 'https://example.com/article' },
      { signal: new AbortController().signal },
    );
    await flushPromises();

    expect(fetchMock).toHaveBeenCalledOnce();
    expect(wrapper.get('#destination-url').element).toHaveProperty('value', 'https://example.com/article');
    expect(wrapper.get('[data-testid="short-link"]').attributes('href'))
      .toBe(`${window.location.origin}/0123456789AB`);
    expect(result).toEqual({
      content: [{ type: 'text', text: `Short link ready: ${window.location.origin}/0123456789AB` }],
    });

    wrapper.unmount();
    expect(registrationSignal?.aborted).toBe(true);
  });
});
