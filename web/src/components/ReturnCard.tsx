import { useState } from 'react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { checkLabel, reasonLabel, redoLabel, scopeLabel, time, usd } from '@/lib/labels'
import type { ReturnRecord } from '@/types/generated/beehive'
import { ProofPanel } from './ProofPanel'

export function ReturnCard({ record }: { record: ReturnRecord }) {
  const [showProof, setShowProof] = useState(false)
  const passed = record.gate === 'pass'

  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between gap-3">
          <CardTitle>
            {record.product_name} · {usd.format(record.amount)}
          </CardTitle>
          <Badge
            className={passed ? 'bg-success/15 text-success' : undefined}
            variant={passed ? 'default' : 'destructive'}
          >
            {passed ? 'Passed' : `Blocked: ${record.blocked_by ? checkLabel[record.blocked_by] : 'gate'}`}
          </Badge>
        </div>
        <CardDescription>
          {record.agent_name} for {record.customer_name} · {reasonLabel[record.reason]} ·{' '}
          {time(record.at)} · <span className="font-mono text-xs">{record.return_id}</span>
        </CardDescription>
      </CardHeader>

      <CardContent className="space-y-2 text-sm">
        <p className={passed ? undefined : 'font-medium text-destructive'}>{record.gate_reason}</p>
        <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
          <dt className="text-muted-foreground">Allowed</dt>
          <dd>
            {record.allowed_scopes.length
              ? record.allowed_scopes.map((scope) => scopeLabel[scope]).join(', ')
              : 'nothing'}{' '}
            · refunds up to {usd.format(record.refund_cap)}
          </dd>
          <dt className="text-muted-foreground">Redo</dt>
          <dd>
            {record.redo ? (
              <>
                {redoLabel[record.redo.status]}{' '}
                <span className="font-mono text-xs text-muted-foreground break-all">{record.redo.return_id}</span>
                {record.redo.label_url && (
                  <>
                    {' · '}
                    <a className="underline underline-offset-4" href={record.redo.label_url} target="_blank" rel="noreferrer">
                      Label
                    </a>
                  </>
                )}
              </>
            ) : (
              'Never reached Redo'
            )}
          </dd>
        </dl>
      </CardContent>

      <CardFooter className="flex-col items-stretch gap-3">
        <Button
          size="lg"
          variant={showProof ? 'outline' : 'default'}
          className="self-start"
          aria-expanded={showProof}
          onClick={() => setShowProof((shown) => !shown)}
        >
          {showProof ? 'Hide proof' : 'Verify proof'}
        </Button>
        {showProof && <ProofPanel returnId={record.return_id} />}
      </CardFooter>
    </Card>
  )
}
