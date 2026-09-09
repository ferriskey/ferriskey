import { useSearchParams } from 'react-router-dom'

/**
 * Ouvre la modale de création quand l'URL porte `?create=1`, et retire le
 * paramètre à la fermeture pour que l'écran ne la rouvre pas au retour
 * arrière. C'est ce qui permet à la page de création d'offrir un « change »
 * qui ramène ici.
 */
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
