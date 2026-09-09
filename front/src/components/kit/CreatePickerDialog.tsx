import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { ChoiceCards, type Choice } from './form'

/**
 * Choix du type d'une ressource avant sa création.
 *
 * FK-29 : la modale vit sur le **listing** et non sur la page de création —
 * c'est depuis la liste qu'on décide de créer, et une page de formulaire
 * masquée derrière une modale bloquante n'a rien à montrer tant qu'on n'a pas
 * choisi. Le type retenu part dans l'URL de création, qui devient donc
 * partageable et rejouable ; un paramètre absent renvoie au listing.
 */
export function CreatePickerDialog<T extends string>({
  open,
  onOpenChange,
  title,
  description,
  options,
  defaultValue,
  /** Reçoit le type choisi et renvoie l'URL de création correspondante. */
  createUrl,
  confirmLabel = 'Continue',
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  description: string
  options: Choice<T>[]
  defaultValue: T
  createUrl: (value: T) => string
  confirmLabel?: string
}) {
  const navigate = useNavigate()
  const [value, setValue] = useState<T>(defaultValue)

  /* Le retour au défaut se fait à la fermeture, dans le gestionnaire qui la
     provoque — pas dans un effet : on n'a rien à synchroniser avec
     l'extérieur, seulement à réagir à un événement. */
  const handleOpenChange = (next: boolean) => {
    if (!next) setValue(defaultValue)
    onOpenChange(next)
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className='sm:max-w-xl'>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>

        <ChoiceCards
          label={title}
          value={value}
          onChange={setValue}
          options={options}
          className='grid-cols-2 max-w-none'
        />

        <DialogFooter>
          <Button variant='ghost' onClick={() => handleOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={() => navigate(createUrl(value))}>{confirmLabel}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
