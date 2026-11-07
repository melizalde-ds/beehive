import { CircleX, RotateCcw } from 'lucide-react'
import { useResetDemo, useReturns } from '@/api/returns'
import { useShop } from '@/api/shop'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { checkLabel } from '@/lib/labels'
import { cn } from '@/lib/utils'
import type { ReturnRecord } from '@/types/generated/beehive'
import { ReturnCard } from './ReturnCard'

export function Dashboard() {
  const shop = useShop()
  const returns = useReturns()
  const reset = useResetDemo()

  return (
    <main className="mx-auto max-w-2xl space-y-4 px-4 py-6">
      <header className="space-y-1">
        <p className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
          <img src="/favicon.svg" alt="" className="size-6" />
          Beehive · shop dashboard
        </p>
        <h1 className="text-2xl font-semibold">{shop.data?.shop.name ?? 'Utah Cache Valley Honey Co.'}</h1>
        <p className="text-sm text-muted-foreground">
          Returns filed by customers' own AI agents. Beehive checks each one, Redo makes the call.
        </p>
        <div className="flex items-center justify-between gap-2">
          <p className="flex items-center gap-2 text-xs text-muted-foreground" aria-live="polite">
            <span className={cn('size-2 rounded-full', returns.live ? 'bg-success' : 'bg-primary')} />
            {returns.live ? 'Live' : 'Refreshing every 3 seconds'}
          </p>
          <Button
            variant="ghost"
            size="xs"
            className="text-muted-foreground"
            disabled={reset.isPending}
            onClick={() => reset.mutate()}
          >
            <RotateCcw />
            {reset.isPending ? 'Resetting…' : 'Reset demo'}
          </Button>
        </div>
        {reset.isError && (
          <p className="text-xs text-destructive" role="alert">
            Couldn't reset the demo. {reset.error.message}
          </p>
        )}
      </header>

      {returns.isPending && <p className="text-muted-foreground">Loading agent returns…</p>}

      {returns.isError && (
        <Alert variant="destructive">
          <CircleX />
          <AlertTitle>Couldn't load the returns</AlertTitle>
          <AlertDescription>
            {returns.error.message} It will try again in a few seconds.
          </AlertDescription>
        </Alert>
      )}

      {returns.data && returns.data.returns.length === 0 && (
        <div className="rounded-xl border border-dashed p-6 text-center text-sm text-muted-foreground">
          No agent has filed a return yet. When one does, it shows up here right away.
        </div>
      )}

      {/* On a phone the third card sits below the fold; this line names every block up front. */}
      {returns.data && returns.data.returns.length > 0 && (
        <BlockSummary returns={returns.data.returns} />
      )}

      {/* The API lists newest first; oldest first reads as the story: pass, cap block, duplicate. */}
      {/* Reset re-issues bh_0001…, so key by filing time too: a re-seeded card remounts with its proof closed. */}
      {returns.data?.returns
        .slice()
        .reverse()
        .map((record) => <ReturnCard key={`${record.return_id}@${record.at}`} record={record} />)}
    </main>
  )
}

function BlockSummary({ returns }: { returns: ReturnRecord[] }) {
  const passed = returns.filter((record) => record.gate === 'pass').length
  // Oldest first, to match the card order.
  const blocked = returns.filter((record) => record.gate === 'block').reverse()
  const reasons = [...new Set(blocked.map((record) => (record.blocked_by ? checkLabel[record.blocked_by] : 'gate')))]
  return (
    <p className="text-sm font-medium">
      <span className="text-success">{passed} passed</span>
      {' · '}
      <span className="text-destructive">
        {blocked.length} blocked{reasons.length > 0 && ` (${reasons.join(', ')})`}
      </span>
    </p>
  )
}
