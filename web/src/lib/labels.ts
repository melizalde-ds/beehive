import type {
  EvidenceStep,
  GateCheck,
  Reason,
  RedoStatus,
  Scope,
} from '../types/generated/beehive'

export const usd = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' })

export const time = (iso: string) =>
  new Date(iso).toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit', second: '2-digit' })

export const reasonLabel: Record<Reason, string> = {
  damaged: 'Arrived damaged',
  wrong_item: 'Wrong item',
  changed_mind: 'Changed mind',
  forgot_cancel: 'Forgot to cancel',
}

export const checkLabel: Record<GateCheck, string> = {
  session: 'Session',
  agent_identity: 'Agent identity',
  scope: 'Scope',
  refund_cap: 'Refund cap',
  expiry: 'Expiry',
  replay: 'Replay',
  idempotency: 'Duplicate',
  rate_limit: 'Rate limit',
  order: 'Order',
}

export const redoLabel: Record<RedoStatus, string> = {
  open: 'Approved',
  in_transit: 'In transit',
  delivered: 'Delivered',
  needs_review: 'Needs review',
  in_review: 'In review',
  complete: 'Complete',
  rejected: 'Denied',
  flagged: 'Flagged',
  pre_shipment: 'Pre-shipment',
  deleted: 'Deleted',
}

export const stepLabel: Record<EvidenceStep, string> = {
  discovery: 'Agent found the shop',
  session: 'Session started',
  consent: 'Customer consent',
  request: 'Return request',
  gate_decision: 'Gate decision',
  redo_result: 'Redo result',
}

export const scopeLabel: Record<Scope, string> = {
  'poppy:read': 'See your orders and returns',
  'poppy:write': 'File returns and claims for you',
}

export const shortHash = (hash: string) => `${hash.slice(0, 8)}…${hash.slice(-6)}`
