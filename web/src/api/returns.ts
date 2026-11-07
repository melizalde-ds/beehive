import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import type { Evidence, ReturnList, TamperRequest, VerifyResult } from '../types/generated/beehive'
import { apiUrl, contractShape, getJson, postJson } from './client'

const ReturnListShape = contractShape<ReturnList>('ReturnList')
const EvidenceShape = contractShape<Evidence>('Evidence')
const VerifyShape = contractShape<VerifyResult>('VerifyResult')

const POLL_MS = 3000
const RECONNECT_MS = 5000

/** Any message on the stream means "the list changed": refetch it. */
function useReturnsStream() {
  const queryClient = useQueryClient()
  const [open, setOpen] = useState(false)

  useEffect(() => {
    let source: EventSource | undefined
    let retry: ReturnType<typeof setTimeout> | undefined
    const connect = () => {
      source = new EventSource(apiUrl('/returns/stream'))
      source.onopen = () => setOpen(true)
      source.onmessage = () => queryClient.invalidateQueries({ queryKey: ['returns'] })
      source.onerror = () => {
        setOpen(false)
        // A non-200 (route not there yet) closes the stream for good, so retry by hand.
        if (source?.readyState === EventSource.CLOSED) retry = setTimeout(connect, RECONNECT_MS)
      }
    }
    connect()
    return () => {
      clearTimeout(retry)
      source?.close()
    }
  }, [queryClient])

  return open
}

/** Live while the stream is open; polls while it's down so the screen never goes stale. */
export function useReturns() {
  const live = useReturnsStream()
  const query = useQuery({
    queryKey: ['returns'],
    queryFn: () => getJson('/returns', ReturnListShape),
    refetchInterval: live ? false : POLL_MS,
  })
  return { ...query, live }
}

export function useEvidence(returnId: string, enabled: boolean) {
  return useQuery({
    queryKey: ['evidence', returnId],
    queryFn: () => getJson(`/evidence/${returnId}`, EvidenceShape),
    enabled,
  })
}

export function useVerify(returnId: string, enabled: boolean) {
  return useQuery({
    queryKey: ['verify', returnId],
    queryFn: () => getJson(`/verify/${returnId}`, VerifyShape),
    enabled,
  })
}

/** Rehearsal only: empties the shop. Return ids restart at bh_0001, so drop cached proofs too. */
export function useResetDemo() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: () => postJson('/demo/reset', {}, ReturnListShape),
    onSuccess: (list) => {
      queryClient.removeQueries({ queryKey: ['evidence'] })
      queryClient.removeQueries({ queryKey: ['verify'] })
      queryClient.setQueryData(['returns'], list)
    },
    onError: (error) => console.error('Demo reset failed', error),
  })
}

export function useTamper(returnId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (body: TamperRequest) =>
      postJson(`/evidence/${returnId}/tamper`, body, EvidenceShape),
    onSuccess: (evidence) => {
      queryClient.setQueryData(['evidence', returnId], evidence)
      return queryClient.invalidateQueries({ queryKey: ['verify', returnId] })
    },
  })
}
