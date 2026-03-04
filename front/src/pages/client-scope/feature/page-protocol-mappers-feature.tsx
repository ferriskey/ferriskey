import { Schemas } from '@/api/api.client'
import { useDeleteProtocolMapper, useGetClientScope } from '@/api/client-scope.api'
import { RouterParams } from '@/routes/router'
import { useMemo, useState } from 'react'
import { useParams } from 'react-router'
import MapperTemplatePickerModalFeature from './modals/mapper-template-picker-modal-feature'
import EditProtocolMapperModalFeature from './modals/edit-protocol-mapper-modal-feature'
import PageProtocolMappers from '../ui/page-protocol-mappers'

import ProtocolMapper = Schemas.ProtocolMapper

export default function PageProtocolMappersFeature() {
  const { realm_name, scope_id } = useParams<RouterParams>()
  const [createOpen, setCreateOpen] = useState(false)
  const [editMapper, setEditMapper] = useState<ProtocolMapper | null>(null)

  const { data: scope, isLoading } = useGetClientScope({
    realm: realm_name ?? 'master',
    scopeId: scope_id,
  })

  const { mutate: deleteMapper } = useDeleteProtocolMapper()

  const mappers = useMemo(() => scope?.protocol_mappers ?? [], [scope])

  const handleDelete = (mapper: ProtocolMapper) => {
    if (!realm_name || !scope_id) return
    deleteMapper({ path: { realm_name, scope_id, mapper_id: mapper.id } })
  }

  return (
    <>
      <PageProtocolMappers
        mappers={mappers}
        isLoading={isLoading}
        onAdd={() => setCreateOpen(true)}
        onEdit={(mapper) => setEditMapper(mapper)}
        onDelete={handleDelete}
      />

      <MapperTemplatePickerModalFeature
        open={createOpen}
        onOpenChange={setCreateOpen}
      />

      <EditProtocolMapperModalFeature
        open={editMapper !== null}
        onOpenChange={(open) => {
          if (!open) setEditMapper(null)
        }}
        mapper={editMapper}
      />
    </>
  )
}
