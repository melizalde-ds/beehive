import { z } from 'zod'
import contract from '../../../schema/beehive.schema.json'
import type { ApiError } from '../types/generated/beehive'

const API_URL: string = import.meta.env.VITE_API_URL || '/api'

export const apiUrl = (path: string) => `${API_URL}${path}`

// Validators are built from the contract itself, so nothing is copied by hand (CT-02).
// The cast is safe: the generated TS type and this validator come from the same schema.
export function contractShape<T>(name: string): z.ZodType<T> {
  return z.fromJSONSchema({
    $schema: 'http://json-schema.org/draft-07/schema#', // draft-07 reads `definitions`
    definitions: contract.definitions,
    $ref: `#/definitions/${name}`,
  }) as z.ZodType<T>
}

async function request<T>(path: string, shape: z.ZodType<T>, init?: RequestInit): Promise<T> {
  const response = await fetch(apiUrl(path), init)
  const body: unknown = await response.json().catch(() => null)
  if (!response.ok) {
    const apiError = body as ApiError | null
    throw new Error(apiError?.message ?? `The Beehive service answered ${response.status}.`)
  }
  const parsed = shape.safeParse(body)
  if (!parsed.success) {
    console.error(`Unexpected response from ${path}`, parsed.error.issues, body)
    throw new Error('Beehive sent something this page does not understand.')
  }
  return parsed.data
}

export const getJson = <T>(path: string, shape: z.ZodType<T>) => request(path, shape)

export const postJson = <T>(path: string, body: unknown, shape: z.ZodType<T>) =>
  request(path, shape, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  })
