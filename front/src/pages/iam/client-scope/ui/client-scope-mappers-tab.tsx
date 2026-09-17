import { useState } from 'react'
import { Link } from 'react-router-dom'
import { Plus, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { MetricsBand, Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import { configToStrings } from '../config-values'
import { templateForType } from '../mapper-templates'
import { FALLBACK_MAPPER_ICON } from '../constants/protocol-mapper-templates'
import { isIdentityMapper, isRoleMapper, mapperCategory } from '../mapper-categories'
import MapperTemplatePickerDialog from './mapper-template-picker-dialog'

import ProtocolMapper = Schemas.ProtocolMapper

const DESTINATIONS = [
  ['id', 'id.token.claim'],
  ['access', 'access.token.claim'],
  ['userinfo', 'userinfo.token.claim'],
] as const

export interface ClientScopeMappersTabProps {
  mappers: ProtocolMapper[]
  mapperHref: (mapper: ProtocolMapper) => string
  pickerOpen: boolean
  onPickerOpenChange: (open: boolean) => void
  onSelectTemplate: (templateId: string) => void
  onDeleteMapper: (mapper: ProtocolMapper) => void
}

export default function ClientScopeMappersTab({
  mappers,
  mapperHref,
  pickerOpen,
  onPickerOpenChange,
  onSelectTemplate,
  onDeleteMapper,
}: ClientScopeMappersTabProps) {
  const { t } = useTranslation('client-scope')
  const [pendingDelete, setPendingDelete] = useState<ProtocolMapper | null>(null)

  const total = mappers.length
  const roleMappers = mappers.filter((m) => isRoleMapper(m.mapper_type)).length
  const identityMappers = mappers.filter((m) => isIdentityMapper(m.mapper_type)).length

  const share = (count: number) =>
    count > 0 && total > 0
      ? t('detail.mappers.metrics.share', { percent: ((count / total) * 100).toFixed(0) })
      : undefined

  return (
    <>
      <MetricsBand
        metrics={[
          {
            key: 'total',
            label: t('detail.mappers.metrics.total.label'),
            value: total,
            hint: t('detail.mappers.metrics.total.hint'),
          },
          {
            key: 'role',
            label: t('detail.mappers.metrics.role.label'),
            value: roleMappers,
            hint: share(roleMappers) ?? t('detail.mappers.metrics.role.empty_hint'),
          },
          {
            key: 'identity',
            label: t('detail.mappers.metrics.identity.label'),
            value: identityMappers,
            hint: share(identityMappers) ?? t('detail.mappers.metrics.identity.empty_hint'),
          },
        ]}
      />

      <Section
        title={t('detail.mappers.title')}
        description={t('detail.mappers.description')}
        action={
          <Button size='sm' onClick={() => onPickerOpenChange(true)}>
            <Plus /> {t('detail.mappers.add')}
          </Button>
        }
        contained={mappers.length > 0}
      >
        {mappers.length > 0 ? (
          <ul className={tokens.surface.divider}>
            {mappers.map((mapper) => {
              const template = templateForType(mapper.mapper_type)
              const config = configToStrings(mapper.config)
              const claim = config['claim.name'] ?? config['token.claim.name']
              const category = mapperCategory(mapper.mapper_type)

              return (
                <li key={mapper.id} className='flex items-center gap-3 py-2.5'>
                  <span className='w-5 shrink-0 text-center text-base leading-none'>
                    {template?.icon ?? FALLBACK_MAPPER_ICON}
                  </span>

                  <div className='min-w-0 flex-1'>
                    <div className='flex flex-wrap items-center gap-2'>
                      <p className='text-xs font-medium text-neutral-900 dark:text-neutral-100'>{mapper.name}</p>
                      <Pill tone={category.tone} mono>
                        {t(category.labelKey)}
                      </Pill>
                      {claim && (
                        <code className='rounded border border-fk-line bg-neutral-50 px-1.5 py-0.5 font-mono-ui text-[11px] text-neutral-600 dark:bg-fk-surface dark:text-neutral-400'>
                          {claim}
                        </code>
                      )}
                    </div>
                    <p className='mt-0.5 truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
                      {mapper.mapper_type}
                    </p>
                  </div>

                  <span className='flex shrink-0 gap-1'>
                    {DESTINATIONS.map(([label, key]) => (
                      <span
                        key={key}
                        className={cn(
                          'rounded px-1 py-0.5 font-mono-ui text-[10px]',
                          config[key] === 'true'
                            ? 'bg-fk-success-soft text-fk-success'
                            : 'bg-neutral-100 text-neutral-300 dark:bg-fk-raised dark:text-neutral-600'
                        )}
                      >
                        {label}
                      </span>
                    ))}
                  </span>

                  <Button variant='ghost' size='sm' className='text-xs' asChild>
                    <Link to={mapperHref(mapper)}>{t('detail.mappers.configure')}</Link>
                  </Button>
                  <Button
                    variant='ghost'
                    size='icon'
                    aria-label={t('detail.mappers.delete_mapper', { name: mapper.name })}
                    onClick={() => setPendingDelete(mapper)}
                    className='size-7 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
                  >
                    <Trash2 />
                  </Button>
                </li>
              )
            })}
          </ul>
        ) : (
          <p className='rounded-sm border border-dashed border-fk-line px-4 py-3 text-xs text-neutral-500 dark:text-neutral-400'>
            {t('detail.mappers.empty')}
          </p>
        )}
      </Section>

      <MapperTemplatePickerDialog
        open={pickerOpen}
        onOpenChange={onPickerOpenChange}
        onSelect={(template) => onSelectTemplate(template.id)}
      />

      <ConfirmDeleteAlert
        open={Boolean(pendingDelete)}
        title={t('detail.mappers.delete.title')}
        description={
          pendingDelete
            ? t('detail.mappers.delete.description', { name: pendingDelete.name })
            : ''
        }
        onConfirm={() => {
          if (!pendingDelete) return
          onDeleteMapper(pendingDelete)
          setPendingDelete(null)
        }}
        onCancel={() => setPendingDelete(null)}
      />
    </>
  )
}
