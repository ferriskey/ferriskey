import { useMemo } from 'react'
import { Schemas } from '@/api/api.client'
import { inferApplicationType } from '../application-types'
import ApplicationQuickstartTab, {
  type QuickstartEndpoint,
} from '../ui/application-quickstart-tab'

import Client = Schemas.Client

export interface ApplicationQuickstartTabFeatureProps {
  application: Client
  realm: string
}

function buildEndpoints(realm: string, includeDevice: boolean): QuickstartEndpoint[] {
  const base = (window.apiUrl ?? '').replace(/\/$/, '')
  const issuer = `${base}/realms/${realm}`

  return [
    {
      key: 'issuer',
      label: 'Issuer',
      description: 'The value your library checks in the iss claim of every token.',
      value: issuer,
    },
    {
      key: 'discovery',
      label: 'Discovery document',
      description: 'Give this one URL to your library and it finds the others by itself.',
      value: `${issuer}/.well-known/openid-configuration`,
    },
    {
      key: 'authorization',
      label: 'Authorization',
      description: 'Where you send the user to sign in.',
      value: `${issuer}/protocol/openid-connect/auth`,
    },
    ...(includeDevice
      ? [
          {
            key: 'device-authorization',
            label: 'Device authorization',
            description: 'Where a browserless device asks for the code it shows to the user.',
            value: `${issuer}/protocol/openid-connect/auth/device`,
          },
        ]
      : []),
    {
      key: 'token',
      label: 'Token',
      description: 'Where the authorization code, or the credentials, are exchanged for tokens.',
      value: `${issuer}/protocol/openid-connect/token`,
    },
    {
      key: 'userinfo',
      label: 'User info',
      description: 'Returns the profile claims for the signed-in user.',
      value: `${issuer}/protocol/openid-connect/userinfo`,
    },
    {
      key: 'jwks',
      label: 'JWKS',
      description: 'The public keys your backend uses to verify a token signature.',
      value: `${issuer}/protocol/openid-connect/jwks.json`,
    },
    {
      key: 'logout',
      label: 'Logout',
      description: 'Ends the FerrisKey session, not only your application session.',
      value: `${issuer}/protocol/openid-connect/logout`,
    },
  ]
}

export default function ApplicationQuickstartTabFeature({
  application,
  realm,
}: ApplicationQuickstartTabFeatureProps) {
  const type = inferApplicationType(application)

  const endpoints = useMemo(() => buildEndpoints(realm, type === 'device'), [realm, type])

  const tokenEndpoint =
    endpoints.find((endpoint) => endpoint.key === 'token')?.value ?? ''
  const authorizationEndpoint =
    endpoints.find((endpoint) => endpoint.key === 'authorization')?.value ?? ''
  const deviceEndpoint =
    endpoints.find((endpoint) => endpoint.key === 'device-authorization')?.value ?? ''

  const firstCallback =
    application.redirect_uris?.[0]?.value ?? 'https://your-app.example.com/callback'

  const snippet =
    type === 'm2m'
      ? `curl -s -X POST '${tokenEndpoint}' \\
  -H 'Content-Type: application/x-www-form-urlencoded' \\
  -d 'grant_type=client_credentials' \\
  -d 'client_id=${application.client_id}' \\
  -d 'client_secret=<YOUR_CLIENT_SECRET>'`
      : type === 'device'
        ? `# 1. Ask for a code and show it to the user
curl -s -X POST '${deviceEndpoint}' \\
  -H 'Content-Type: application/x-www-form-urlencoded' \\
  -d 'client_id=${application.client_id}'

# 2. Poll until the user has approved it
curl -s -X POST '${tokenEndpoint}' \\
  -H 'Content-Type: application/x-www-form-urlencoded' \\
  -d 'grant_type=urn:ietf:params:oauth:grant-type:device_code' \\
  -d 'client_id=${application.client_id}' \\
  -d 'device_code=<DEVICE_CODE>'`
        : `# 1. Send the user to the authorization endpoint
${authorizationEndpoint}?response_type=code
  &client_id=${application.client_id}
  &redirect_uri=${encodeURIComponent(firstCallback)}
  &scope=openid%20profile%20email

# 2. Exchange the returned code for tokens
curl -s -X POST '${tokenEndpoint}' \\
  -H 'Content-Type: application/x-www-form-urlencoded' \\
  -d 'grant_type=authorization_code' \\
  -d 'client_id=${application.client_id}' \\
  -d 'redirect_uri=${firstCallback}' \\
  -d 'code=<AUTHORIZATION_CODE>'`

  const snippetTitle =
    type === 'm2m'
      ? 'Get a token with the client credentials'
      : type === 'device'
        ? 'Get a token with the device flow'
        : 'Sign a user in with the authorization code flow'

  const snippetDescription =
    type === 'm2m'
      ? 'No user is involved: the application authenticates as itself and receives an access token.'
      : type === 'device'
        ? 'The device shows a code, the user approves it on another screen, then the device gets its tokens.'
        : 'Redirect the user to sign in, then exchange the returned code for tokens on your side.'

  return (
    <ApplicationQuickstartTab
      clientId={application.client_id}
      endpoints={endpoints}
      snippetTitle={snippetTitle}
      snippetDescription={snippetDescription}
      snippet={snippet}
    />
  )
}
