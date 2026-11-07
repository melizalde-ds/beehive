import { ConsentScreen } from './components/ConsentScreen'
import { Dashboard } from './components/Dashboard'

// Two screens, no router: the contract's /consent/:session_id, or ?consent=<id>.
function consentSessionId() {
  const fromPath = window.location.pathname.match(/^\/consent\/([^/]+)/)?.[1]
  return fromPath ?? new URLSearchParams(window.location.search).get('consent')
}

export default function App() {
  const sessionId = consentSessionId()
  return sessionId ? <ConsentScreen sessionId={decodeURIComponent(sessionId)} /> : <Dashboard />
}
