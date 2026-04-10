import { useSearchParams } from 'react-router'
import PageRequiredAction from '../ui/page-required-action'

export default function PageRequiredActionFeature() {
  const [searchParams] = useSearchParams()
  const execution = searchParams.get('execution')
  const token = searchParams.get('client_data')

  if (!token) {
    return <div>Loading ...</div>
  }

  return (
    <div>
      <PageRequiredAction execution={execution ?? ''} />
    </div>
  )
}
