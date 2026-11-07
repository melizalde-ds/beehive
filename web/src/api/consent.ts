import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { ConsentDecision, SessionResponse } from '../types/generated/beehive'
import { contractShape, getJson, postJson } from './client'

const SessionShape = contractShape<SessionResponse>('SessionResponse')

export function useSession(sessionId: string) {
  return useQuery({
    queryKey: ['session', sessionId],
    queryFn: () => getJson(`/poppy/session/${sessionId}`, SessionShape),
  })
}

export function useConsent(sessionId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (decision: ConsentDecision) =>
      postJson(`/consent/${sessionId}`, decision, SessionShape),
    onSuccess: (session) => queryClient.setQueryData(['session', sessionId], session),
  })
}
