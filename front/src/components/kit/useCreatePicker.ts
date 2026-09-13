import { useSearchParams } from 'react-router-dom'

export function useCreatePicker() {
  const [params, setParams] = useSearchParams()
  const open = params.get('create') === '1'

  const setOpen = (next: boolean) => {
    const updated = new URLSearchParams(params)
    if (next) updated.set('create', '1')
    else updated.delete('create')
    setParams(updated, { replace: true })
  }

  return { open, setOpen }
}
