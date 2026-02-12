import type { Ability } from '../actor.js';

export interface ApiResponse<T = unknown> {
  ok: boolean;
  status: number;
  data: T;
}

const DEFAULT_TIMEOUT_MS = 30_000;

export class CallApi implements Ability {
  private constructor(
    private readonly baseUrl: string,
    private readonly timeoutMs: number,
  ) {}

  static at(baseUrl: string, timeoutMs = DEFAULT_TIMEOUT_MS): CallApi {
    return new CallApi(baseUrl, timeoutMs);
  }

  async get<T = unknown>(path: string): Promise<ApiResponse<T>> {
    const res = await fetch(`${this.baseUrl}${path}`, {
      signal: AbortSignal.timeout(this.timeoutMs),
    });
    return this.parseResponse<T>(res);
  }

  async post<T = unknown>(path: string, body?: unknown): Promise<ApiResponse<T>> {
    const res = await fetch(`${this.baseUrl}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: body !== undefined ? JSON.stringify(body) : undefined,
      signal: AbortSignal.timeout(this.timeoutMs),
    });
    return this.parseResponse<T>(res);
  }

  private async parseResponse<T>(res: Response): Promise<ApiResponse<T>> {
    const data = res.headers.get('content-type')?.includes('application/json')
      ? await res.json()
      : await res.text();
    return { ok: res.ok, status: res.status, data: data as T };
  }
}
