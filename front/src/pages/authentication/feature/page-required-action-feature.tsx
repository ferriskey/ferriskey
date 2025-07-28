import { useParams, useSearchParams } from 'react-router'
import PageRequiredAction from '../ui/page-required-action'
import { useAuthenticateMutation } from '@/api/auth.api'
import { RouterParams } from '@/routes/router'
import { useEffect } from 'react'
export default function PageRequiredActionFeature() {
  const [searchParams] = useSearchParams()
  const { realm_name } = useParams<RouterParams>()
  const {
    mutate: authenticate,
    data: authenticateData,
    status: authenticateStatus,
  } = useAuthenticateMutation()

  const execution = searchParams.get('execution')
  const token = searchParams.get('client_data')

  const handle = () => {
    const cookies = document.cookie.split(';').reduce(
      (acc, cookie) => {
        const [key, value] = cookie.trim().split('=')
        acc[key] = value
        return acc
      },
      {} as Record<string, string>
    )

    const sessionCode = cookies['FERRISKEY_SESSION'] || ''
    authenticate({
      clientId: 'security-admin-console',
      realm: realm_name ?? 'master',
      data: {},
      sessionCode: sessionCode,
      useToken: true,
      token: token ?? undefined,
    })
  }

  useEffect(() => {
    if (authenticateData && authenticateData.url) {
      window.location.href = authenticateData.url
    }
  }, [authenticateData])

  return (
    <div>
      <PageRequiredAction execution={execution ?? ''} />
    </div>
  )
}
