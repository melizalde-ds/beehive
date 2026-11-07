import { useQuery } from '@tanstack/react-query'
import type { Seed } from '../types/generated/beehive'
import { contractShape, getJson } from './client'

const SeedShape = contractShape<Seed>('Seed')

export function useShop() {
  return useQuery({ queryKey: ['shop'], queryFn: () => getJson('/shop', SeedShape) })
}
