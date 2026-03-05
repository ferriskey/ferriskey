import { Schemas } from '@/api/api.client'
import { useDeleteProtocolMapper, useGetClientScope } from '@/api/client-scope.api'
import { RouterParams } from '@/routes/router'
import { useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import MapperTemplatePickerModalFeature from './modals/mapper-template-picker-modal-feature'
import PageProtocolMappers from '../ui/page-protocol-mappers'
import {
  CLIENT_SCOPE_MAPPER_SETTINGS_URL,
  CLIENT_SCOPE_URL,
} from '@/routes/sub-router/client-scope.router'

import ProtocolMapper = Schemas.ProtocolMapper

interface ConfirmState {
  title: string
  description: string
  open: boolean
  onConfirm: () => void
}

const DEFAULT_CONFIRM: ConfirmState = {
  title: '',
  description: '',
  open: false,
  onConfirm: () => {},
}

export default function PageProtocolMappersFeature() {
  const { realm_name, scope_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const [createOpen, setCreateOpen] = useState(false)
  const [confirm, setConfirm] = useState<ConfirmState>(DEFAULT_CONFIRM)

  const { data: scope, isLoading } = useGetClientScope({
    realm: realm_name ?? 'master',
    scopeId: scope_id,
  })

  const { mutate: deleteMapper } = useDeleteProtocolMapper()

  const mappers = useMemo(() => scope?.protocol_mappers ?? [], [scope])

  const statistics = useMemo(() => {
    const total = mappers.length
    const roleMappers = mappers.filter((m) =>
      m.mapper_type.toLowerCase().includes('role')
    ).length
    const identityMappers = mappers.filter((m) => {
      const t = m.mapper_type.toLowerCase()
      return (
        t.includes('usermodel') ||
        t.includes('attribute') ||
        t.includes('property') ||
        t.includes('full-name')
      )
    }).length
    return { total, roleMappers, identityMappers }
  }, [mappers])

  const handleClickRow = (mapper: ProtocolMapper) => {
    navigate(
      `${CLIENT_SCOPE_URL(realm_name, scope_id)}${CLIENT_SCOPE_MAPPER_SETTINGS_URL(mapper.id)}`
    )
  }

  const handleDelete = (mapper: ProtocolMapper) => {
    setConfirm({
      title: 'Delete protocol mapper',
      description: `Are you sure you want to delete "${mapper.name}"? This action cannot be undone.`,
      open: true,
      onConfirm: () => {
        if (!realm_name || !scope_id) return
        deleteMapper({ path: { realm_name, scope_id, mapper_id: mapper.id } })
        setConfirm(DEFAULT_CONFIRM)
      },
    })
  }

  return (
    <>
      <PageProtocolMappers
        mappers={mappers}
        isLoading={isLoading}
        statistics={statistics}
        confirm={confirm}
        onConfirmClose={() => setConfirm(DEFAULT_CONFIRM)}
        onAdd={() => setCreateOpen(true)}
        onClickRow={handleClickRow}
        onDelete={handleDelete}
      />

      <MapperTemplatePickerModalFeature
        open={createOpen}
        onOpenChange={setCreateOpen}
      />
    </>
  )
}
