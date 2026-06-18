import { Schemas } from '@/api/api.client'
import { Check, Copy } from 'lucide-react'
import { useMemo, useState } from 'react'
import { inferApplicationType } from '../../types'
import { CopyRow, Section } from './primitives'

import Client = Schemas.Client

interface Props {
  client: Client
  realm: string
}

function buildEndpoints(realm: string) {
  // window.apiUrl is the resolved API origin (set during app bootstrap).
  const base = (window.apiUrl ?? '').replace(/\/$/, '')
  const issuer = `${base}/realms/${realm}`
  return {
    issuer,
    discovery: `${issuer}/.well-known/openid-configuration`,
    authorization: `${issuer}/protocol/openid-connect/auth`,
    token: `${issuer}/protocol/openid-connect/token`,
    userinfo: `${issuer}/protocol/openid-connect/userinfo`,
    jwks: `${issuer}/protocol/openid-connect/certs`,
    logout: `${issuer}/protocol/openid-connect/logout`,
  }
}

export default function QuickstartTab({ client, realm }: Props) {
  const endpoints = useMemo(() => buildEndpoints(realm), [realm])
  const appType = inferApplicationType(client)
  const isM2M = appType === 'm2m'

  const firstRedirect = client.redirect_uris?.[0]?.value ?? 'https://your-app.example.com/callback'

  const snippet = isM2M
    ? `curl -s -X POST '${endpoints.token}' \\
  -H 'Content-Type: application/x-www-form-urlencoded' \\
  -d 'grant_type=client_credentials' \\
  -d 'client_id=${client.client_id}' \\
  -d 'client_secret=<YOUR_CLIENT_SECRET>'`
    : `# 1. Send the user to the authorization endpoint
${endpoints.authorization}?response_type=code
  &client_id=${client.client_id}
  &redirect_uri=${encodeURIComponent(firstRedirect)}
  &scope=openid%20profile%20email

# 2. Exchange the returned code for tokens
curl -s -X POST '${endpoints.token}' \\
  -H 'Content-Type: application/x-www-form-urlencoded' \\
  -d 'grant_type=authorization_code' \\
  -d 'client_id=${client.client_id}' \\
  -d 'redirect_uri=${firstRedirect}' \\
  -d 'code=<AUTHORIZATION_CODE>'`

  return (
    <div className='flex flex-col gap-6'>
      <Section
        title='OpenID Connect endpoints'
        description={`Point your ${isM2M ? 'service' : 'app'}'s OIDC/OAuth library at these URLs.`}
      >
        <EndpointRow label='Issuer' value={endpoints.issuer} />
        <EndpointRow label='Discovery document' value={endpoints.discovery} />
        <EndpointRow label='Authorization' value={endpoints.authorization} />
        <EndpointRow label='Token' value={endpoints.token} />
        <EndpointRow label='User info' value={endpoints.userinfo} />
        <EndpointRow label='JWKS' value={endpoints.jwks} />
        <EndpointRow label='Logout' value={endpoints.logout} />
      </Section>

      <Section title='Client ID'>
        <CopyRow value={client.client_id} />
      </Section>

      <Section
        title={isM2M ? 'Get a token (client credentials)' : 'Authorization Code flow'}
        description={
          isM2M
            ? 'Exchange your client credentials directly for an access token.'
            : 'Redirect the user to authorize, then swap the code for tokens on your server.'
        }
      >
        <CodeBlock code={snippet} />
      </Section>
    </div>
  )
}

function EndpointRow({ label, value }: { label: string; value: string }) {
  return (
    <div className='flex flex-col gap-1.5'>
      <span className='text-xs font-medium text-muted-foreground'>{label}</span>
      <CopyRow value={value} />
    </div>
  )
}

function CodeBlock({ code }: { code: string }) {
  const [copied, setCopied] = useState(false)
  const copy = () => {
    void navigator.clipboard.writeText(code)
    setCopied(true)
    window.setTimeout(() => setCopied(false), 1500)
  }
  return (
    <div className='relative'>
      <pre className='overflow-x-auto rounded-md border border-border bg-muted/40 p-4 text-xs font-mono leading-relaxed'>
        {code}
      </pre>
      <button
        type='button'
        onClick={copy}
        className='absolute right-2 top-2 inline-flex h-7 w-7 items-center justify-center rounded-md border border-border bg-background text-muted-foreground hover:text-foreground hover:bg-muted transition-colors'
        aria-label='Copy snippet'
      >
        {copied ? <Check className='h-3.5 w-3.5 text-emerald-500' /> : <Copy className='h-3.5 w-3.5' />}
      </button>
    </div>
  )
}
