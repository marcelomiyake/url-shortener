import type { WebMCP } from 'webmcp-types';

import type { ShortLink } from './links';

type CreateLink = (destinationUrl: string, signal: AbortSignal) => Promise<ShortLink>;

const createShortLinkSchema = {
  type: 'object',
  properties: {
    destinationUrl: {
      type: 'string',
      description: 'The HTTP or HTTPS destination address to shorten.',
    },
  },
  required: ['destinationUrl'],
  additionalProperties: false,
} as const;

export interface ToolRegistration {
  ready: Promise<void>;
  unregister: () => void;
}

export function registerShortLinkTool(
  modelContext: WebMCP.ModelContext | undefined,
  createLink: CreateLink,
  pageOrigin: string,
): ToolRegistration | null {
  if (!modelContext || typeof modelContext.registerTool !== 'function') return null;

  const registration = new AbortController();
  const ready = modelContext.registerTool({
    name: 'create-short-link',
    title: 'Create a short link',
    description: 'Creates or reuses a public, non-expiring short link for an HTTP or HTTPS destination. Creating the mapping persists it in the URL shortener.',
    inputSchema: createShortLinkSchema,
    annotations: {
      readOnlyHint: false,
      consequentialHint: true,
    },
    async execute({ destinationUrl }, { signal }) {
      if (typeof destinationUrl !== 'string' || destinationUrl.trim().length === 0) {
        throw new Error('Provide an HTTP or HTTPS destination address.');
      }

      const shortLink = await createLink(destinationUrl, signal);
      const shortUrl = new URL(shortLink.short_path, pageOrigin);
      if (shortUrl.origin !== pageOrigin) {
        throw new Error('The service returned an invalid short link. Try again later.');
      }

      return {
        content: [{
          type: 'text',
          text: `Short link ready: ${shortUrl.href}`,
        }],
      };
    },
  }, { signal: registration.signal }).catch(() => {
    registration.abort();
  });

  return {
    ready,
    unregister: () => registration.abort(),
  };
}
