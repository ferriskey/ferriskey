import { useState } from 'react'

export function useDraft<T>(key: string, pristine: T) {
  const [draft, setDraft] = useState<{ key: string; value: T }>({ key, value: pristine })

  if (draft.key !== key) setDraft({ key, value: pristine })

  const value = draft.key === key ? draft.value : pristine

  return {
    value,
    patch: (next: Partial<T>) => setDraft({ key, value: { ...value, ...next } }),
    reset: () => setDraft({ key, value: pristine }),
  }
}
