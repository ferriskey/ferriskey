import { useSearchParams } from 'react-router'
import PageRequiredAction from '../ui/page-required-action'
import { useAuth } from '@/hooks/use-auth'
import { useEffect } from 'react'

export default function PageRequiredActionFeature() {
  const [searchParams] = useSearchParams()
  const { setAuthToken } = useAuth()
  const execution = searchParams.get('execution')
  const token = searchParams.get('client_data')

  useEffect(() => {
    if (token) {
      setAuthToken(token)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token])

  if (!token) {
    return <div>Loading ...</div>
  }

  return (
    <div>
      <PageRequiredAction execution={execution ?? ''} />
    </div>
  )
}
