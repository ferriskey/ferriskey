import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import {
  MapperTemplate,
  QUICK_START_TEMPLATES,
  MAPPER_CATALOG,
} from '../../constants/protocol-mapper-templates'
import {
  CLIENT_SCOPE_MAPPER_CREATE_URL,
  CLIENT_SCOPE_URL,
} from '@/routes/sub-router/client-scope.router'
import MapperTemplatePickerModal from '../../ui/modals/mapper-template-picker-modal'

interface MapperTemplatePickerModalFeatureProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export default function MapperTemplatePickerModalFeature({
  open,
  onOpenChange,
}: MapperTemplatePickerModalFeatureProps) {
  const { realm_name, scope_id } = useParams<RouterParams>()
  const navigate = useNavigate()

  const handleSelectTemplate = (template: MapperTemplate) => {
    onOpenChange(false)
    navigate(
      `${CLIENT_SCOPE_URL(realm_name, scope_id)}${CLIENT_SCOPE_MAPPER_CREATE_URL}?template=${template.id}`
    )
  }

  return (
    <MapperTemplatePickerModal
      open={open}
      onOpenChange={onOpenChange}
      quickStartTemplates={QUICK_START_TEMPLATES}
      catalogTemplates={MAPPER_CATALOG}
      onSelectTemplate={handleSelectTemplate}
    />
  )
}
