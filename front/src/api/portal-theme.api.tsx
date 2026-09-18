import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import type { ReactNode } from 'react'
import { BaseQuery } from '.'
import { preloadNamespaces, translate } from '@/lib/i18n'
import type { Schemas } from './api.client'

const PORTAL_NAMESPACE = 'portal'

void preloadNamespaces(PORTAL_NAMESPACE).catch(() => undefined)

// ---------- Legacy single-theme-per-realm hooks (kept until cleanup PR) ----------

export const useGetPortalTheme = ({ realm = 'master' }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal/theme', {
      path: { realm_name: realm },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useUpdatePortalTheme = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/portal/theme').mutationOptions,
    onSuccess: async (_, variables) => {
      const themeKeys = window.tanstackApi.get('/realms/{realm_name}/portal/theme', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey

      const loginSettingsKeys = window.tanstackApi.get('/realms/{name}/login-settings', {
        path: { name: variables.path.realm_name },
      }).queryKey

      await Promise.all([
        queryClient.invalidateQueries({ queryKey: themeKeys }),
        queryClient.invalidateQueries({ queryKey: loginSettingsKeys }),
      ])

      toast.success(translate('portal:themes.toast.saved'))
    },
  })
}

// ---------- Collection API ----------

export const useListPortalThemes = ({ realm = 'master' }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal/themes', {
      path: { realm_name: realm },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useGetPortalThemeById = ({
  realm = 'master',
  themeId,
}: BaseQuery & { themeId: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal/themes/{theme_id}', {
      path: { realm_name: realm, theme_id: themeId },
    }).queryOptions,
    enabled: !!realm && !!themeId && themeId !== 'new',
  })
}

export const useCreatePortalTheme = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/portal/themes').mutationOptions,
    onSuccess: async (_, variables) => {
      const listKey = window.tanstackApi.get('/realms/{realm_name}/portal/themes', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: listKey }),
        queryClient.invalidateQueries({
          queryKey: activeThemeQueryKey(variables.path.realm_name),
        }),
      ])
      toast.success(translate('portal:themes.toast.created'))
    },
  })
}

// Metadata + page mutations skip per-call toasts: the theme builder fires
// both in parallel under one "Save theme" action, and reports a single
// aggregated toast so a 422 on the page tree never reads as success.
export const useUpdatePortalThemeMetadata = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/portal/themes/{theme_id}')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      await invalidateThemeQueries(queryClient, variables.path.realm_name, variables.path.theme_id)
    },
  })
}

export const useImportPortalTheme = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/portal/themes/import')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const listKey = window.tanstackApi.get('/realms/{realm_name}/portal/themes', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey
      await queryClient.invalidateQueries({ queryKey: listKey })
      toast.success(translate('portal:themes.toast.imported'))
    },
    // The server refuses a file that belongs to another builder or that uses a
    // format version it cannot read; its message says which, so surface it.
    onError: (error) => {
      toast.error(translate('portal:themes.toast.import_failed'), { description: error.message })
    },
  })
}

export const useUpdatePortalThemePage = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'put',
      '/realms/{realm_name}/portal/themes/{theme_id}/pages/{page_type}',
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      await invalidateThemeQueries(queryClient, variables.path.realm_name, variables.path.theme_id)
    },
  })
}

function labelForBlockType(type: string): string {
  return type.replace(/_/g, ' ')
}

function labelForPageType(pageType: Schemas.PortalPageType): string {
  return translate(`${PORTAL_NAMESPACE}:page_type.${pageType}.label`)
}

// Custom fetcher (see src/api/index.ts) throws an Error with the parsed body
// on `.data`, NOT on `.response.data`. The backend wraps each issue in
// `errors: [{ message, field }]`; for portal page validation the `message` is
// itself a JSON-encoded `MissingBlocks` payload that we decode here.
interface PortalApiError {
  data?: { errors?: { message?: string; field?: string }[] }
  message?: string
}

interface MissingBlocksPayload {
  page_type?: string
  missing?: string[]
}

function readApiErrors(error: unknown): { message?: string; field?: string }[] {
  if (!error || typeof error !== 'object') return []
  const errors = (error as PortalApiError).data?.errors
  return Array.isArray(errors) ? errors : []
}

function parseMissingBlocks(message: string): MissingBlocksPayload | null {
  try {
    const parsed = JSON.parse(message) as MissingBlocksPayload
    if (parsed && Array.isArray(parsed.missing)) return parsed
  } catch {
    /* not a structured payload */
  }
  return null
}

/** One structured issue extracted from the API error payload. */
export interface PortalPageIssue {
  pageType?: Schemas.PortalPageType
  missing: string[]
  /** Raw message when the payload couldn't be parsed as `MissingBlocks`. */
  raw?: string
}

/**
 * Decodes the API error into structured issues. Each `errors[]` entry from
 * the backend becomes one item — for portal validation that's one item per
 * page tree that failed required-block checks.
 */
export function extractPortalPageIssues(error: unknown): PortalPageIssue[] {
  const apiErrors = readApiErrors(error)
  const issues: PortalPageIssue[] = []
  for (const { message } of apiErrors) {
    if (typeof message !== 'string') continue
    const missingBlocks = parseMissingBlocks(message)
    if (missingBlocks?.missing?.length) {
      issues.push({
        pageType: missingBlocks.page_type as Schemas.PortalPageType | undefined,
        missing: missingBlocks.missing,
      })
      continue
    }
    issues.push({ missing: [], raw: message })
  }
  return issues
}

/**
 * Produces a single-line human-readable description for a failed portal page
 * save or activation, used wherever the toast description must be a string.
 * Returns `null` only when no message can be derived at all.
 */
export function describePortalPageError(error: unknown): string | null {
  const issues = extractPortalPageIssues(error)
  if (issues.length === 0) {
    return error instanceof Error ? error.message : null
  }
  const parts = issues.map((issue) => {
    if (issue.missing.length > 0) {
      const blocks = issue.missing.map(labelForBlockType).join(', ')
      const count = issue.missing.length
      return issue.pageType
        ? translate('portal:page_builder.error.missing_on_page', {
            count,
            blocks,
            page: labelForPageType(issue.pageType),
          })
        : translate('portal:page_builder.error.missing', { count, blocks })
    }
    return issue.raw ?? translate('portal:page_builder.toast.unknown_error')
  })
  return parts.length > 0 ? parts.join('\n') : null
}

/**
 * Renders a tidy multi-line description for toasts: one bullet per affected
 * page, blocks comma-separated. Falls back to a plain string when no
 * structured payload was available.
 */
function renderPortalIssuesDescription(error: unknown): ReactNode {
  const issues = extractPortalPageIssues(error)
  if (issues.length === 0) {
    return describePortalPageError(error) ?? translate('portal:page_builder.toast.unknown_error')
  }
  return (
    <div className='flex flex-col gap-1'>
      <span>{translate('portal:page_builder.error.pages_missing_blocks')}</span>
      <ul className='ml-4 list-disc space-y-0.5'>
        {issues.map((issue, idx) => {
          const pageLabel = issue.pageType
            ? labelForPageType(issue.pageType)
            : translate('portal:page_builder.error.unknown_page')
          const blocks =
            issue.missing.length > 0
              ? issue.missing.map(labelForBlockType).join(', ')
              : (issue.raw ?? translate('portal:page_builder.toast.unknown_error'))
          return (
            <li key={`${pageLabel}-${idx}`}>
              <span className='font-medium'>{pageLabel}</span>: {blocks}
            </li>
          )
        })}
      </ul>
    </div>
  )
}

export const useActivatePortalTheme = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/portal/themes/{theme_id}/activate',
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      await invalidateThemeQueries(queryClient, variables.path.realm_name, variables.path.theme_id)
      toast.success(translate('portal:themes.toast.activated'))
    },
    onError: (error) => {
      toast.error(translate('portal:themes.toast.activation_failed'), {
        description: renderPortalIssuesDescription(error),
      })
    },
  })
}

export const useDeletePortalTheme = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/portal/themes/{theme_id}')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const listKey = window.tanstackApi.get('/realms/{realm_name}/portal/themes', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey
      // Deleting the live theme changes which one is active, so the active
      // query has to go too.
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: listKey }),
        queryClient.invalidateQueries({
          queryKey: activeThemeQueryKey(variables.path.realm_name),
        }),
      ])
      toast.success(translate('portal:themes.toast.deleted'))
    },
  })
}

// ---------- Public renderer + introspection ----------

export const useGetActivePortalTheme = ({
  realm = 'master',
  pageType,
  enabled = true,
}: BaseQuery & { pageType: Schemas.PortalPageType; enabled?: boolean }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal/active', {
      path: { realm_name: realm },
      query: { page_type: pageType },
    }).queryOptions,
    enabled: enabled && !!realm && !!pageType,
  })
}

export const useGetPortalPageRequirements = ({ realm = 'master' }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal/page-requirements', {
      path: { realm_name: realm },
    }).queryOptions,
    enabled: !!realm,
  })
}

/**
 * Which theme is active is its own query, not a field on the list.
 *
 * `/portal/active` is keyed by page type, and the themes list carries no
 * `active` flag — so anything that changes which theme is live has to expire
 * this query by hand, or the console keeps showing the previous winner until
 * something else happens to refetch it.
 *
 * The key is built without `query` on purpose: React Query matches keys
 * partially, so omitting the page type expires every cached page type at once.
 */
function activeThemeQueryKey(realmName: string) {
  return [{ _id: '/realms/{realm_name}/portal/active', path: { realm_name: realmName } }]
}

async function invalidateThemeQueries(
  queryClient: ReturnType<typeof useQueryClient>,
  realmName: string,
  themeId: string,
) {
  const listKey = window.tanstackApi.get('/realms/{realm_name}/portal/themes', {
    path: { realm_name: realmName },
  }).queryKey
  const itemKey = window.tanstackApi.get('/realms/{realm_name}/portal/themes/{theme_id}', {
    path: { realm_name: realmName, theme_id: themeId },
  }).queryKey

  await Promise.all([
    queryClient.invalidateQueries({ queryKey: listKey }),
    queryClient.invalidateQueries({ queryKey: itemKey }),
    queryClient.invalidateQueries({ queryKey: activeThemeQueryKey(realmName) }),
  ])
}
