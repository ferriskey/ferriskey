import type { RelationSource } from './RelationSelect'

export type ColumnFilterField =
  | { kind: 'text'; key: string; label: string; placeholder?: string }
  | { kind: 'boolean'; key: string; label: string }
  | { kind: 'enum'; key: string; label: string; options: { value: string; label: string }[] }
  | { kind: 'relation'; key: string; label: string; relation: RelationSource }
  | { kind: 'date-range'; fromKey: string; toKey: string; label: string }

export type ColumnFilterIndicator =
  | { kind: 'none' }
  | { kind: 'count'; count: number }
  | { kind: 'value'; key: string; value: 'true' | 'false' }
  | { kind: 'active' }

export function fieldKeys(field: ColumnFilterField): string[] {
  return field.kind === 'date-range' ? [field.fromKey, field.toKey] : [field.key]
}

function isActive(field: ColumnFilterField, filters: Record<string, string>): boolean {
  return fieldKeys(field).some((key) => (filters[key] ?? '') !== '')
}

export function activeFieldCount(fields: ColumnFilterField[], filters: Record<string, string>): number {
  return fields.filter((field) => isActive(field, filters)).length
}

export function columnFilterIndicator(
  fields: ColumnFilterField[],
  filters: Record<string, string>,
): ColumnFilterIndicator {
  const active = fields.filter((field) => isActive(field, filters))
  if (active.length === 0) return { kind: 'none' }
  if (active.length > 1) return { kind: 'count', count: active.length }
  const [field] = active
  if (field.kind === 'boolean') {
    const value = filters[field.key]
    if (value === 'true' || value === 'false') return { kind: 'value', key: field.key, value }
  }
  return { kind: 'active' }
}

export function countActiveFilters(
  columns: { filters?: ColumnFilterField[] }[],
  filters: Record<string, string>,
): number {
  return columns.reduce((total, column) => total + activeFieldCount(column.filters ?? [], filters), 0)
}

export function clearColumnFilters(fields: ColumnFilterField[]): Record<string, string> {
  return Object.fromEntries(fields.flatMap(fieldKeys).map((key) => [key, '']))
}
