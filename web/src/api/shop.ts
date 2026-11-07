import { useQuery } from '@tanstack/react-query'
import type { Seed } from '../types/generated/beehive'
import { getJson } from './client'

export function useShop() {
  return useQuery({ queryKey: ['shop'], queryFn: () => getJson<Seed>('/shop') })
}
