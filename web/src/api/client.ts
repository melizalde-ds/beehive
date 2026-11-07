import type { ApiError } from '../types/generated/beehive'

const API_URL: string = import.meta.env.VITE_API_URL || '/api'

// shortcut: trusts the server to match the generated type; add zod checks in phase 05.
export async function getJson<T>(path: string): Promise<T> {
  const response = await fetch(`${API_URL}${path}`)
  const body: unknown = await response.json().catch(() => null)
  if (!response.ok) {
    const apiError = body as ApiError | null
    throw new Error(apiError?.message ?? `The Beehive service answered ${response.status}.`)
  }
  return body as T
}
