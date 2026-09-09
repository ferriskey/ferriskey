import type { Schemas } from '@/api/api.client'
import type { BuilderNode } from '@/lib/builder-core'
import { PORTAL_PAGES, type PortalPageType } from './portal-pages'

export interface PortalPageStatus {
  pageType: PortalPageType
  required: string[]
  present: string[]
  missing: string[]
}

function snakeToCamel(value: string): string {
  return value.replace(/_([a-z])/g, (_, c: string) => c.toUpperCase())
}

export function readPageTree(
  theme: Schemas.PortalTheme | undefined,
  pageType: PortalPageType
): unknown {
  const pages = theme?.pages as Record<string, unknown> | undefined
  return pages?.[snakeToCamel(pageType)] ?? []
}

export function parseTree(tree: unknown): BuilderNode[] {
  if (Array.isArray(tree)) return tree as BuilderNode[]
  if (tree && typeof tree === 'object' && Array.isArray((tree as { children?: unknown }).children)) {
    return (tree as { children: BuilderNode[] }).children
  }
  return []
}

function collectTypes(value: unknown, acc: Set<string>) {
  if (Array.isArray(value)) {
    value.forEach((v) => collectTypes(v, acc))
    return
  }
  if (value && typeof value === 'object') {
    const obj = value as Record<string, unknown>
    if (typeof obj.type === 'string') acc.add(obj.type)
    Object.values(obj).forEach((v) => collectTypes(v, acc))
  }
}

export function countNodes(tree: unknown): number {
  let total = 0
  const walk = (value: unknown) => {
    if (Array.isArray(value)) {
      value.forEach(walk)
      return
    }
    if (value && typeof value === 'object') {
      const obj = value as Record<string, unknown>
      if (typeof obj.type === 'string') total += 1
      Object.values(obj).forEach(walk)
    }
  }
  walk(tree)
  return total
}

export type RequirementsByPage = Partial<Record<PortalPageType, string[]>>

export function requirementsByPage(
  requirements: Schemas.PageRequirement[] | undefined
): RequirementsByPage {
  const map: RequirementsByPage = {}
  for (const entry of requirements ?? []) {
    map[entry.page_type] = entry.required_blocks
  }
  return map
}

export function statusesForTheme(
  theme: Schemas.PortalTheme | undefined,
  requirements: RequirementsByPage
): PortalPageStatus[] {
  return PORTAL_PAGES.map(({ type }) => {
    const required = requirements[type] ?? []
    const acc = new Set<string>()
    collectTypes(readPageTree(theme, type), acc)
    const present = required.filter((block) => acc.has(block))
    return {
      pageType: type,
      required,
      present,
      missing: required.filter((block) => !acc.has(block)),
    }
  })
}

export function failingPages(statuses: PortalPageStatus[]): PortalPageStatus[] {
  return statuses.filter((s) => s.missing.length > 0)
}
