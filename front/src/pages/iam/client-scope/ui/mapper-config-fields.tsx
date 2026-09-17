import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { FieldRow, SwitchField } from '@/components/kit'
import type { ConfigFieldDef } from '@/pages/iam/client-scope/constants/protocol-mapper-templates'
import { MAPPER_ENTITY_FIELDS, type MapperEntityKind } from '../config-values'
import MapperEntityField, { type MapperEntityOption } from './mapper-entity-field'

export type MapperEntityOptions = Record<MapperEntityKind, MapperEntityOption[]>

const entityCopyKeys: Record<
  MapperEntityKind,
  { placeholder: string; searchPlaceholder: string; emptyLabel: string }
> = {
  client: {
    placeholder: 'mapper_entity.client.placeholder',
    searchPlaceholder: 'mapper_entity.client.search_placeholder',
    emptyLabel: 'mapper_entity.client.empty',
  },
  role: {
    placeholder: 'mapper_entity.role.placeholder',
    searchPlaceholder: 'mapper_entity.role.search_placeholder',
    emptyLabel: 'mapper_entity.role.empty',
  },
}

export interface MapperConfigFieldsProps {
  fields: ConfigFieldDef[]
  values: Record<string, string>
  entityOptions: MapperEntityOptions
  onChange: (key: string, value: string) => void
}

export default function MapperConfigFields({
  fields,
  values,
  entityOptions,
  onChange,
}: MapperConfigFieldsProps) {
  const { t } = useTranslation('client-scope')

  return (
    <>
      {fields.map((field) => {
        const id = `mapper-config-${field.key.replace(/\./g, '-')}`
        const value = values[field.key] ?? field.defaultValue ?? ''
        const entityKind = MAPPER_ENTITY_FIELDS[field.key]

        return (
          <FieldRow
            key={field.key}
            label={t(field.labelKey)}
            description={field.descriptionKey ? t(field.descriptionKey) : undefined}
            htmlFor={id}
          >
            {field.type === 'switch' ? (
              <SwitchField
                id={id}
                checked={value === 'true'}
                onCheckedChange={(checked) => onChange(field.key, String(checked))}
              />
            ) : field.type === 'select' ? (
              <Select value={value} onValueChange={(v) => onChange(field.key, v)}>
                <SelectTrigger id={id} className='w-full max-w-sm'>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {field.options?.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {t(option.labelKey)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : entityKind ? (
              <MapperEntityField
                id={id}
                options={entityOptions[entityKind]}
                value={value}
                placeholder={t(entityCopyKeys[entityKind].placeholder)}
                searchPlaceholder={t(entityCopyKeys[entityKind].searchPlaceholder)}
                emptyLabel={t(entityCopyKeys[entityKind].emptyLabel)}
                onChange={(next) => onChange(field.key, next)}
              />
            ) : (
              <Input
                id={id}
                value={value}
                placeholder={field.placeholder}
                onChange={(e) => onChange(field.key, e.target.value)}
                className='max-w-sm'
              />
            )}
            <p className='mt-1.5 font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{field.key}</p>
          </FieldRow>
        )
      })}
    </>
  )
}
