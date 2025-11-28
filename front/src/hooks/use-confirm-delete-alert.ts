import { useState } from 'react'

export type ConfirmOptions = {
  title: string
  description: string
  onConfirm: () => void
}

export function useConfirmDeleteAlert() {
  const [confirm, setConfirm] = useState<ConfirmOptions & { open: boolean }>({
    open: false,
    title: '',
    description: '',
    onConfirm: () => {},
  })

  const ask = (options: ConfirmOptions) => {
    setConfirm({ open: true, ...options })
  }

  const close = () => setConfirm((prev) => ({ ...prev, open: false }))

  return {
    confirm,
    ask,
    close,
  }
}
