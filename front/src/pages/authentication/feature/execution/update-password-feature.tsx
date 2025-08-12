import { useParams, useSearchParams } from 'react-router'
import { RouterParams } from '@/routes/router.ts'
import UpdatePassword from '@/pages/authentication/ui/execution/update-password.tsx'

export default function UpdatePasswordFeature() {
  const { realm_name } = useParams<RouterParams>()
  const [searchParams] = useSearchParams()
  const token = searchParams.get('client_data')


  return (
    <UpdatePassword />
  )
}