import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import type { Schemas } from '@/api/api.client'
import { buildDocumentTitle, type RealmLabel } from '@/lib/i18n/document-title'

export function usePortalDocumentTitle(
  pageType: Schemas.PortalPageType,
  realm: RealmLabel | null
) {
  const { t } = useTranslation('auth')
  const title = buildDocumentTitle(t(`document_title.${pageType}`), realm)

  useEffect(() => {
    const previous = document.title
    document.title = title

    return () => {
      document.title = previous
    }
  }, [title])
}
