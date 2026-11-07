import { CircleCheck, CircleX } from 'lucide-react'
import { useState } from 'react'
import { useConsent, useSession } from '@/api/consent'
import { useShop } from '@/api/shop'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { scopeLabel, time, usd } from '@/lib/labels'
import type { SessionResponse } from '@/types/generated/beehive'

export function ConsentScreen({ sessionId }: { sessionId: string }) {
  const session = useSession(sessionId)
  const shopName = useShop().data?.shop.name ?? 'Utah Cache Valley Honey Co.'

  return (
    <main className="mx-auto max-w-md px-4 py-8">
      <p className="mb-4 flex items-center gap-2 text-sm font-medium text-muted-foreground">
        <img src="/favicon.svg" alt="" className="size-6" />
        Beehive
      </p>
      {session.isPending && <p className="text-muted-foreground">Loading the request…</p>}
      {session.isError && (
        <Alert variant="destructive">
          <CircleX />
          <AlertTitle>Couldn't find this request</AlertTitle>
          <AlertDescription>
            {session.error.message} Ask your agent to start again.
          </AlertDescription>
        </Alert>
      )}
      {session.data && <ConsentCard session={session.data} shopName={shopName} />}
    </main>
  )
}

function ConsentCard({ session, shopName }: { session: SessionResponse; shopName: string }) {
  const consent = useConsent(session.session_id)
  const [allowWrite, setAllowWrite] = useState(session.scopes.includes('poppy:write'))
  const [cap, setCap] = useState(String(session.refund_cap))

  if (session.status === 'active') {
    return (
      <Alert className="border-success/40 text-success">
        <CircleCheck />
        <AlertTitle>Approved</AlertTitle>
        <AlertDescription className="text-success/90">
          Your agent can act for you at {shopName}
          {session.scopes.includes('poppy:write') && `, with refunds up to ${usd.format(session.refund_cap)}`}
          {session.expires_at && ` until ${time(session.expires_at)}`}. You can close this tab.
        </AlertDescription>
      </Alert>
    )
  }
  if (session.status === 'denied') {
    return (
      <Alert>
        <CircleX />
        <AlertTitle>Denied</AlertTitle>
        <AlertDescription>{shopName} won't take requests from your agent. You can close this tab.</AlertDescription>
      </Alert>
    )
  }

  const capValue = Number(cap)
  // The customer may lower the cap, never raise it past what the agent asked for.
  const capValid = cap !== '' && capValue >= 0 && capValue <= session.refund_cap
  const scopes = session.scopes.filter((scope) => scope !== 'poppy:write' || allowWrite)

  return (
    <Card>
      <CardHeader>
        <p className="text-sm font-medium text-muted-foreground">{shopName}</p>
        <CardTitle className="text-xl">Your AI agent is asking to act for you</CardTitle>
        <CardDescription>It can only ask. Beehive checks every request against what you allow here.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <ul className="space-y-3">
          {session.scopes.map((scope) => (
            <li key={scope} className="flex items-center gap-3">
              {scope === 'poppy:write' ? (
                <>
                  <Checkbox
                    id="allow-write"
                    checked={allowWrite}
                    onCheckedChange={(checked) => setAllowWrite(checked === true)}
                  />
                  <Label htmlFor="allow-write">{scopeLabel[scope]}</Label>
                </>
              ) : (
                <>
                  <CircleCheck className="size-4 text-success" aria-hidden />
                  <span className="text-sm">{scopeLabel[scope]}</span>
                </>
              )}
            </li>
          ))}
        </ul>
        {allowWrite && (
          <div className="space-y-1.5">
            <Label htmlFor="refund-cap">Refund cap (USD)</Label>
            <Input
              id="refund-cap"
              type="number"
              inputMode="decimal"
              min={0}
              max={session.refund_cap}
              step="0.01"
              value={cap}
              onChange={(event) => setCap(event.target.value)}
              aria-invalid={!capValid}
              className="h-11 text-base"
            />
            <p className="text-xs text-muted-foreground">
              Any return over this amount is blocked. Up to {usd.format(session.refund_cap)}.
            </p>
          </div>
        )}
        {consent.isError && (
          <p role="alert" className="text-sm text-destructive">
            Couldn't save your answer. {consent.error.message}
          </p>
        )}
      </CardContent>
      <CardFooter className="flex-col gap-2 sm:flex-row">
        <Button
          className="h-11 w-full sm:flex-1"
          disabled={consent.isPending || (allowWrite && !capValid)}
          onClick={() => consent.mutate({ approve: true, scopes, refund_cap: allowWrite ? capValue : 0 })}
        >
          {consent.isPending ? 'Saving…' : allowWrite ? `Approve, cap ${capValid ? usd.format(capValue) : '…'}` : 'Approve read only'}
        </Button>
        <Button
          variant="outline"
          className="h-11 w-full sm:flex-1"
          disabled={consent.isPending}
          onClick={() => consent.mutate({ approve: false, scopes: [], refund_cap: 0 })}
        >
          Deny
        </Button>
      </CardFooter>
    </Card>
  )
}
