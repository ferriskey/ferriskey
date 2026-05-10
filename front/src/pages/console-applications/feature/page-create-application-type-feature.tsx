import { RouterParams } from '@/routes/router'
import {
  APPLICATIONS_URL,
  APPLICATION_CREATE_TYPE_URL,
  ApplicationType,
} from '@/routes/sub-router/applications.router'
import { useNavigate, useParams } from 'react-router'
import PageCreateApplicationType from '../ui/page-create-application-type'

export default function PageCreateApplicationTypeFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()

  const handleCancel = () => {
    if (!realm_name) return
    navigate(APPLICATIONS_URL(realm_name))
  }

  const handlePick = (type: ApplicationType) => {
    if (!realm_name) return
    navigate(APPLICATION_CREATE_TYPE_URL(realm_name, type))
  }

  return <PageCreateApplicationType onCancel={handleCancel} onPick={handlePick} />
}
