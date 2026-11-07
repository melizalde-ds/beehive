import { useEffect, useRef } from 'react'
import { CircleCheck, CircleX } from 'lucide-react'
import { useEvidence, useTamper, useVerify } from '@/api/returns'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import { checkLabel, shortHash, stepLabel, time } from '@/lib/labels'
import type { GateCheck } from '@/types/generated/beehive'

const summary = (data: object) =>
  Object.entries(data)
    .map(([key, value]) => `${key}: ${typeof value === 'object' ? JSON.stringify(value) : String(value)}`)
    .join(' · ')

type CheckRow = { check: GateCheck; ok: boolean }

const isCheckRow = (row: unknown): row is CheckRow =>
  typeof row === 'object' &&
  row !== null &&
  'check' in row &&
  'ok' in row &&
  typeof row.check === 'string' &&
  row.check in checkLabel &&
  typeof row.ok === 'boolean'

/** The gate decision link, as one row per check instead of raw JSON. */
function GateChecks({ data }: { data: object }) {
  const checks = 'checks' in data && Array.isArray(data.checks) ? data.checks.filter(isCheckRow) : []
  const reason = 'reason' in data && typeof data.reason === 'string' ? data.reason : null
  if (!checks.length) return <p className="mt-1 break-words text-muted-foreground">{summary(data)}</p>
  return (
    <div className="mt-1 space-y-1">
      <ul className="grid grid-cols-2 gap-x-3 gap-y-0.5">
        {checks.map((row) => (
          <li key={row.check} className={cn('flex items-center gap-1.5', row.ok ? 'text-success' : 'text-destructive')}>
            {row.ok ? <CircleCheck className="size-3.5 shrink-0" /> : <CircleX className="size-3.5 shrink-0" />}
            <span>
              {checkLabel[row.check]}
              <span className="sr-only">{row.ok ? ' passed' : ' blocked'}</span>
            </span>
          </li>
        ))}
      </ul>
      {reason && <p className="break-words text-muted-foreground">{reason}</p>}
    </div>
  )
}

export function ProofPanel({ returnId }: { returnId: string }) {
  const verify = useVerify(returnId, true)
  const evidence = useEvidence(returnId, true)
  const tamper = useTamper(returnId)
  const verdictRef = useRef<HTMLDivElement>(null)
  const broken = verify.data?.ok === false

  // After the tamper the chain grows taller and the verdict scrolls off; bring it back.
  useEffect(() => {
    if (broken) verdictRef.current?.scrollIntoView({ behavior: 'smooth', block: 'center' })
  }, [broken])

  if (verify.isPending || evidence.isPending) {
    return <p className="text-sm text-muted-foreground">Checking every link in the chain…</p>
  }
  if (verify.isError || evidence.isError) {
    console.error(verify.error ?? evidence.error)
    return (
      <Alert variant="destructive">
        <CircleX />
        <AlertTitle>Couldn't check the proof</AlertTitle>
        <AlertDescription>{(verify.error ?? evidence.error)?.message}</AlertDescription>
      </Alert>
    )
  }

  const result = verify.data
  const requestLink = evidence.data.links.find((link) => link.step === 'request')

  return (
    <div className="space-y-3">
      <Alert
        ref={verdictRef}
        variant={result.ok ? 'default' : 'destructive'}
        className={cn(result.ok && 'border-success/40 text-success')}
        aria-live="polite"
      >
        {result.ok ? <CircleCheck /> : <CircleX />}
        <AlertTitle>{result.ok ? 'Proof checks out' : 'Proof is broken'}</AlertTitle>
        <AlertDescription className={cn(result.ok && 'text-success/90')}>
          {result.message}
        </AlertDescription>
      </Alert>

      <ol className="space-y-2">
        {evidence.data.links.map((link) => {
          const broken = result.broken_at === link.index
          return (
            <li
              key={link.index}
              className={cn(
                'rounded-lg border p-2.5 text-sm',
                broken ? 'border-destructive bg-destructive/5' : 'border-border bg-muted/40',
              )}
            >
              <div className="flex flex-wrap items-baseline justify-between gap-x-3">
                <span className="font-medium">
                  {link.index}. {stepLabel[link.step]}
                  {broken && <span className="text-destructive"> (changed)</span>}
                </span>
                <span className="text-xs text-muted-foreground">{time(link.at)}</span>
              </div>
              {link.step === 'gate_decision' ? (
                <GateChecks data={link.data} />
              ) : (
                <p className="mt-1 break-words text-muted-foreground">{summary(link.data)}</p>
              )}
              <p className="mt-1 font-mono text-xs text-muted-foreground">{shortHash(link.hash)}</p>
            </li>
          )
        })}
      </ol>

      {requestLink && (
        <div className="space-y-1">
          <Button
            variant="outline"
            size="lg"
            disabled={tamper.isPending}
            onClick={() => tamper.mutate({ link_index: requestLink.index })}
          >
            {tamper.isPending ? 'Editing…' : 'Edit one byte (demo)'}
          </Button>
          <p className="text-xs text-muted-foreground">
            Changes one digit in the stored return request, without re-hashing it.
          </p>
          {tamper.isError && (
            <p role="alert" className="text-sm text-destructive">
              Couldn't edit the link. {tamper.error.message}
            </p>
          )}
        </div>
      )}
    </div>
  )
}
