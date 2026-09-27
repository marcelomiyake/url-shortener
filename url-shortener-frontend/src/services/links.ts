export interface ShortLink {
  code: string;
  short_path: string;
}

export class LinkApiError extends Error {
  constructor(
    message: string,
    readonly code: string,
  ) {
    super(message);
    this.name = 'LinkApiError';
  }
}

export async function createShortLink(
  destinationUrl: string,
  signal?: AbortSignal,
): Promise<ShortLink> {
  let response: Response;
  try {
    const request: RequestInit = {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ destination_url: destinationUrl }),
    };
    if (signal) request.signal = signal;

    response = await fetch('/api/v1/links', request);
  } catch {
    if (signal?.aborted) {
      throw new LinkApiError(
        'The request was cancelled before a response arrived. The link may already exist; retrying the same URL will reuse it.',
        'request_cancelled',
      );
    }
    throw new LinkApiError('The service could not be reached. Try again later.', 'network_error');
  }

  const payload: unknown = await response.json().catch(() => null);
  if (!response.ok) {
    const error = readApiError(payload);
    throw new LinkApiError(error.message, error.code);
  }

  if (!isShortLink(payload)) {
    throw new LinkApiError('The service returned an unexpected response. Try again later.', 'invalid_response');
  }
  return payload;
}

function readApiError(payload: unknown): { code: string; message: string } {
  if (typeof payload !== 'object' || payload === null || !('error' in payload)) {
    return { code: 'request_failed', message: 'The short link could not be created. Try again later.' };
  }

  const details = payload.error;
  if (
    typeof details === 'object'
    && details !== null
    && 'code' in details
    && 'message' in details
    && typeof details.code === 'string'
    && typeof details.message === 'string'
  ) {
    return { code: details.code, message: details.message };
  }
  return { code: 'request_failed', message: 'The short link could not be created. Try again later.' };
}

function isShortLink(payload: unknown): payload is ShortLink {
  if (typeof payload !== 'object' || payload === null || !('code' in payload) || !('short_path' in payload)) {
    return false;
  }

  const { code, short_path: shortPath } = payload;
  return typeof code === 'string'
    && /^[0-9A-Za-z]{12,43}$/.test(code)
    && typeof shortPath === 'string'
    && shortPath === `/${code}`;
}
