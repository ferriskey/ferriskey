import { authStore } from '@/store/auth.store.ts'

/**
 * Downloading an export cannot go through the generated client: it hands back a
 * parsed body, while a download needs the bytes and the filename the server
 * chose in `Content-Disposition`. So this issues the request itself, with the
 * same bearer token the generated fetcher uses.
 */
async function fetchExport(path: string): Promise<{ blob: Blob; filename: string }> {
  const token = authStore.getState().accessToken
  const response = await fetch(new URL(path, window.apiUrl).toString(), {
    headers: token ? { Authorization: `Bearer ${token}` } : undefined,
  })

  if (!response.ok) {
    throw new Error(`export failed with status ${response.status}`)
  }

  return {
    blob: await response.blob(),
    filename: filenameFromContentDisposition(response.headers.get('Content-Disposition')),
  }
}

/** Falls back to a neutral name when the header is missing or unparseable. */
function filenameFromContentDisposition(header: string | null): string {
  const match = header?.match(/filename="([^"]+)"/)
  return match?.[1] ?? 'export.json'
}

/** Hands the blob to the browser as a save-to-disk, then releases the object URL. */
function saveBlob(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  link.remove()
  URL.revokeObjectURL(url)
}

export async function downloadEmailTemplateExport(
  realm: string,
  templateId: string,
  format: 'json' | 'mjml',
): Promise<void> {
  const { blob, filename } = await fetchExport(
    `/realms/${realm}/email-templates/${templateId}/export?format=${format}`,
  )
  saveBlob(blob, filename)
}

export async function downloadPortalLayoutExport(realm: string, layoutId: string): Promise<void> {
  const { blob, filename } = await fetchExport(
    `/realms/${realm}/portal-layouts/${layoutId}/export`,
  )
  saveBlob(blob, filename)
}

export async function downloadPortalThemeExport(realm: string, themeId: string): Promise<void> {
  const { blob, filename } = await fetchExport(
    `/realms/${realm}/portal/themes/${themeId}/export`,
  )
  saveBlob(blob, filename)
}

/** Reads a file the user picked and parses it as the JSON export envelope. */
export async function readExportFile(file: File): Promise<Record<string, unknown>> {
  const text = await file.text()

  try {
    return JSON.parse(text) as Record<string, unknown>
  } catch {
    throw new Error('This file is not valid JSON.')
  }
}
