import { afterEach, describe, expect, it, vi } from 'vitest';
import { API_URL, apiGet } from './api';

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('apiGet', () => {
  it('calls the api origin and returns the JSON body', async () => {
    const fetchMock = vi.fn(async () => new Response(JSON.stringify({ status: 'ok' })));
    vi.stubGlobal('fetch', fetchMock);

    await expect(apiGet<{ status: string }>('/health')).resolves.toEqual({ status: 'ok' });
    expect(fetchMock).toHaveBeenCalledWith(`${API_URL}/health`, undefined);
  });

  it('throws with the path and status on a non-2xx answer', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => new Response('{}', { status: 503 })),
    );

    await expect(apiGet('/health')).rejects.toThrow('/health → 503');
  });
});
