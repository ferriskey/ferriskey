import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

/* -------------------------------------------------------------- */
/* Pill — badge sémantique de la charte FerrisKey                   */
/* -------------------------------------------------------------- */

export type PillTone =
  | 'neutral'
  | 'success'
  | 'info'
  | 'violet'
  | 'amber'
  | 'danger'
  | 'primary'

/* FK-02 : chaque teinte existe en triplet — texte plein, fond -soft,
   bordure -border. Une pastille qui n'utilise que la teinte pleine sur
   fond blanc ne se distingue pas d'un simple texte coloré.
   FK-30 : l'état se lit en forme et en couleur, jamais en couleur seule. */
const pillTones: Record<PillTone, string> = {
  neutral: 'border-fk-line bg-neutral-50 text-neutral-600',
  success: 'border-fk-success-border bg-fk-success-soft text-fk-success',
  info: 'border-fk-info-border bg-fk-info-soft text-fk-info',
  violet: 'border-fk-violet-border bg-fk-violet-soft text-fk-violet',
  amber: 'border-fk-amber-border bg-fk-amber-soft text-fk-amber',
  danger: 'border-fk-danger-border bg-fk-danger-soft text-fk-danger',
  primary: 'border-fk-primary-border bg-fk-primary-soft text-fk-primary-text',
}

export function Pill({
  tone = 'neutral',
  mono,
  className,
  children,
}: {
  tone?: PillTone
  /** FK-04 : réservé aux valeurs littérales — identifiants, clés, URL. */
  mono?: boolean
  className?: string
  children: ReactNode
}) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 rounded border px-1.5 py-0.5 text-xs leading-5',
        mono && 'font-mono-ui',
        pillTones[tone],
        className
      )}
    >
      {children}
    </span>
  )
}

export function StatusDot({ on }: { on: boolean }) {
  return (
    <span
      className={cn(
        'inline-block size-1.5 rounded-full',
        on ? 'bg-fk-success' : 'bg-neutral-300'
      )}
    />
  )
}

/* -------------------------------------------------------------- */
/* Squircle — la vignette carrée à initiale des listes             */
/* -------------------------------------------------------------- */

const squircleTones = [
  'bg-fk-brand',
  'bg-fk-primary',
  'bg-fk-info',
  'bg-fk-success',
  'bg-fk-violet',
  'bg-fk-amber',
]

export function Squircle({
  name,
  size = 'md',
  className,
}: {
  name: string
  /**
   * FK-09 : `xl` (60 px) est calé sur la hauteur d'un en-tête de ressource à
   * deux lignes — 28 px de titre + 6 px d'écart + 26 px de pastille. Taille
   * figée plutôt que dérivée : en flex, `aspect-square` ne sait pas remonter
   * une largeur depuis une hauteur posée par `align-self: stretch`.
   *
   * L'initiale occupe ~40 % du carré à chaque taille, pour que la vignette se
   * lise pareil d'une ligne de liste à un en-tête de page.
   */
  size?: 'sm' | 'md' | 'lg' | 'xl'
  className?: string
}) {
  const tone =
    squircleTones[
      name.split('').reduce((a, c) => a + c.charCodeAt(0), 0) %
        squircleTones.length
    ]

  return (
    <span
      className={cn(
        'grid shrink-0 place-items-center rounded-md font-semibold uppercase text-white',
        size === 'sm' && 'size-6 text-[11px]',
        size === 'md' && 'size-9 text-sm',
        size === 'lg' && 'size-11 text-base',
        size === 'xl' && 'size-15 text-2xl',
        tone,
        className
      )}
    >
      {name.charAt(0)}
    </span>
  )
}

/* -------------------------------------------------------------- */
/* Titres                                                          */
/* -------------------------------------------------------------- */

export function Eyebrow({ children }: { children: ReactNode }) {
  return (
    <p className='text-[11px] font-semibold uppercase tracking-[0.14em] text-neutral-400'>
      {children}
    </p>
  )
}

/* -------------------------------------------------------------- */
/* IconTile — la vignette d'icône teintée                          */
/* -------------------------------------------------------------- */

const iconTileTones = {
  info: 'bg-fk-info-soft text-fk-info',
  success: 'bg-fk-success-soft text-fk-success',
  violet: 'bg-fk-violet-soft text-fk-violet',
  amber: 'bg-fk-amber-soft text-fk-amber',
  danger: 'bg-fk-danger-soft text-fk-danger',
  primary: 'bg-fk-primary-soft text-fk-primary-text',
} as const

export function IconTile({
  tone,
  children,
  className,
}: {
  tone: keyof typeof iconTileTones
  children: ReactNode
  className?: string
}) {
  return (
    <span
      className={cn(
        'grid size-9 shrink-0 place-items-center rounded-md',
        iconTileTones[tone],
        className
      )}
    >
      {children}
    </span>
  )
}
