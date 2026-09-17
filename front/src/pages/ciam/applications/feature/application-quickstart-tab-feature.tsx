import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { Schemas } from '@/api/api.client'
import { inferApplicationType, type ConsoleTranslate } from '../application-types'
import ApplicationQuickstartTab, {
  type QuickstartEndpoint,
} from '../ui/application-quickstart-tab'

import Client = Schemas.Client

export interface ApplicationQuickstartTabFeatureProps {
  application: Client
  realm: string
}

const DEVICE_ENDPOINT_KEY = 'device-authorization'
const TOKEN_ENDPOINT_KEY = 'token'
const AUTHORIZATION_ENDPOINT_KEY = 'authorization'

const FALLBACK_CALLBACK = 'https://your-app.example.com/callback'

function buildEndpoints(
  realm: string,
  includeDevice: boolean,
  t: ConsoleTranslate
): QuickstartEndpoint[] {
  const base = (window.apiUrl ?? '').replace(/\/$/, '')
  const issuer = `${base}/realms/${realm}`

  const describe = (catalog: string) => ({
    label: t(`applications.quickstart.endpoints.${catalog}.label`),
    description: t(`applications.quickstart.endpoints.${catalog}.description`),
  })

  return [
    {
      key: 'issuer',
      ...describe('issuer'),
      value: issuer,
    },
    {
      key: 'discovery',
      ...describe('discovery'),
      value: `${issuer}/.well-known/openid-configuration`,
    },
    {
      key: AUTHORIZATION_ENDPOINT_KEY,
      ...describe('authorization'),
      value: `${issuer}/protocol/openid-connect/auth`,
    },
    ...(includeDevice
      ? [
          {
            key: DEVICE_ENDPOINT_KEY,
            ...describe('device_authorization'),
            value: `${issuer}/protocol/openid-connect/auth/device`,
          },
        ]
      : []),
    {
      key: TOKEN_ENDPOINT_KEY,
      ...describe('token'),
      value: `${issuer}/protocol/openid-connect/token`,
    },
    {
      key: 'userinfo',
      ...describe('userinfo'),
      value: `${issuer}/protocol/openid-connect/userinfo`,
    },
    {
      key: 'jwks',
      ...describe('jwks'),
      value: `${issuer}/protocol/openid-connect/jwks.json`,
    },
    {
      key: 'logout',
      ...describe('logout'),
      value: `${issuer}/protocol/openid-connect/logout`,
    },
  ]
}

export default function ApplicationQuickstartTabFeature({
  application,
  realm,
}: ApplicationQuickstartTabFeatureProps) {
  const { t } = useTranslation('console')
  const type = inferApplicationType(application)

  const endpoints = useMemo(
    () => buildEndpoints(realm, type === 'device', t),
    [realm, type, t]
  )

  const tokenEndpoint =
    endpoints.find((endpoint) => endpoint.key === TOKEN_ENDPOINT_KEY)?.value ?? ''
  const authorizationEndpoint =
    endpoints.find((endpoint) => endpoint.key === AUTHORIZATION_ENDPOINT_KEY)?.value ?? ''
  const deviceEndpoint =
    endpoints.find((endpoint) => endpoint.key === DEVICE_ENDPOINT_KEY)?.value ?? ''

  const firstCallback = application.redirect_uris?.[0]?.value ?? FALLBACK_CALLBACK

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

  const snippetCatalog =
    type === 'm2m' ? 'm2m' : type === 'device' ? 'device' : 'authorization_code'

  const snippetTitle = t(`applications.quickstart.snippet.${snippetCatalog}.title`)
  const snippetDescription = t(`applications.quickstart.snippet.${snippetCatalog}.description`)

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
