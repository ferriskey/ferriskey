import { useSearchParams } from 'react-router'
import PageRequiredAction from '../ui/page-required-action'
import { useAuth } from '@/hooks/use-auth'
import { useEffect, useState } from 'react'

export default function PageRequiredActionFeature() {
  const [searchParams] = useSearchParams()
  const { setAuthToken } = useAuth()
  const execution = searchParams.get('execution')
  const token = searchParams.get('client_data')

  const [isTokenSet, setIsTokenSet] = useState(false)

  useEffect(() => {
    if (token && !isTokenSet) {
      setIsTokenSet(true)
    }
  }, [token, setAuthToken, isTokenSet])

  if (!token && !isTokenSet) {
    return <div>Loading ...</div>
  }

  return (
    <div>
      <PageRequiredAction execution={execution ?? ''} />
    </div>
  )
}
