import type { Ability } from '../actor.js';

export interface ApiResponse<T = unknown> {
  ok: boolean;
  status: number;
  data: T;
}

export class CallApi implements Ability {
  private constructor(private readonly baseUrl: string) {}

  static at(baseUrl: string): CallApi {
    return new CallApi(baseUrl);
  }

  async get<T = unknown>(path: string): Promise<ApiResponse<T>> {
    const res = await fetch(`${this.baseUrl}${path}`);
    const data = res.headers.get('content-type')?.includes('application/json')
      ? await res.json()
      : await res.text();
    return { ok: res.ok, status: res.status, data: data as T };
  }

  async post<T = unknown>(path: string, body?: unknown): Promise<ApiResponse<T>> {
    const res = await fetch(`${this.baseUrl}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
    const data = res.headers.get('content-type')?.includes('application/json')
      ? await res.json()
      : await res.text();
    return { ok: res.ok, status: res.status, data: data as T };
  }

  async put<T = unknown>(path: string, body?: unknown): Promise<ApiResponse<T>> {
    const res = await fetch(`${this.baseUrl}${path}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
    const data = res.headers.get('content-type')?.includes('application/json')
      ? await res.json()
      : await res.text();
    return { ok: res.ok, status: res.status, data: data as T };
  }

  async delete<T = unknown>(path: string): Promise<ApiResponse<T>> {
    const res = await fetch(`${this.baseUrl}${path}`, { method: 'DELETE' });
    const data = res.headers.get('content-type')?.includes('application/json')
      ? await res.json()
      : await res.text();
    return { ok: res.ok, status: res.status, data: data as T };
  }
}
