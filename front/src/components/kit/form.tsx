import { useState, type ComponentType, type ReactNode } from 'react'
import { Check, Plus, X } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import { Button } from '@/components/ui/button'

/* -------------------------------------------------------------- */
/* FieldRow — la ligne de formulaire unique de la console           */
/* -------------------------------------------------------------- */

/**
 * Libellé et description à gauche sur 20 rem, contrôle à droite.
 *
 * Remplace le motif `flex items-start justify-between` avec `w-1/3` / `w-1/2`
 * recopié dans toute la console. FK-21 : la description a sa propre ligne et
 * n'est jamais tronquée — c'est précisément la phrase qui évite de configurer
 * le champ de travers. FK-24 : pleine largeur plutôt que deux colonnes
 * serrées, qui écrasent la colonne de libellés.
 */
export function FieldRow({
  label,
  description,
  htmlFor,
  children,
  className,
  layout = 'split',
}: {
  label: string
  /** FK-22 : dit la conséquence, pas la définition. */
  description?: string
  htmlFor?: string
  children: ReactNode
  className?: string
  /** `stacked` pour les formulaires étroits (modale, tiroir latéral). */
  layout?: 'split' | 'stacked'
}) {
  if (layout === 'stacked') {
    return (
      <div className={cn('space-y-1.5', className)}>
        <label
          htmlFor={htmlFor}
          className='block text-sm font-medium text-neutral-900'
        >
          {label}
        </label>
        {description && <p className='text-xs text-neutral-500'>{description}</p>}
        <div className='pt-0.5'>{children}</div>
      </div>
    )
  }

  return (
    <div
      className={cn(
        'grid gap-x-8 gap-y-2 py-4 md:grid-cols-[minmax(0,20rem)_minmax(0,1fr)]',
        className
      )}
    >
      <div className='min-w-0'>
        <label
          htmlFor={htmlFor}
          className='block text-sm font-medium text-neutral-900'
        >
          {label}
        </label>
        {description && (
          <p className='mt-0.5 text-xs leading-relaxed text-neutral-500'>
            {description}
          </p>
        )}
      </div>
      <div className='min-w-0'>{children}</div>
    </div>
  )
}

/* -------------------------------------------------------------- */
/* SwitchField — FK-27 : l'interrupteur survit aux vrais booléens   */
/* -------------------------------------------------------------- */

/**
 * « Use TLS », « Standard flow », « Scheduled sync » : une capacité qu'on
 * active. Pas de carte pour ça — la carte coûte trois fois la hauteur. Dès
 * que les deux états sont deux *natures*, c'est {@link ChoiceCards} (FK-26).
 */
export function SwitchField({
  checked,
  onCheckedChange,
  onLabel = 'Enabled',
  offLabel = 'Disabled',
  disabled,
  id,
}: {
  checked: boolean
  onCheckedChange: (v: boolean) => void
  onLabel?: string
  offLabel?: string
  disabled?: boolean
  id?: string
}) {
  return (
    <label
      className={cn(
        'inline-flex items-center gap-2.5',
        disabled ? 'cursor-not-allowed opacity-60' : 'cursor-pointer'
      )}
    >
      <Switch
        id={id}
        checked={checked}
        disabled={disabled}
        onCheckedChange={onCheckedChange}
      />
      <span
        className={cn('text-sm', checked ? 'text-neutral-900' : 'text-neutral-500')}
      >
        {checked ? onLabel : offLabel}
      </span>
    </label>
  )
}

/* -------------------------------------------------------------- */
/* ChoiceCards — un choix parmi N, présenté en cartes jointes       */
/* -------------------------------------------------------------- */

export interface Choice<T extends string> {
  value: T
  label: string
  /** Ce que le choix implique concrètement, en une phrase. */
  description?: string
  icon?: ComponentType<{ className?: string; strokeWidth?: number }>
  /** FK-39 : un refus du domaine s'annonce avant le clic, avec sa raison. */
  disabledReason?: string
}

/**
 * FK-26 : alternative à l'interrupteur quand les deux états ne sont pas
 * « actif / inactif » mais deux options de même rang — confidentiel et
 * public, temporaire et permanent, expirer et terminer. Le switch obligeait à
 * deviner l'état opposé ; ici les deux sont nommés et décrits côte à côte.
 *
 * Géométrie : cartes jointes comme les segments d'un input group — une seule
 * bordure partagée (`-ml-px`), les arrondis seulement aux extrémités du
 * groupe. La carte retenue passe en `z-10` pour que sa bordure accentuée
 * couvre celle de sa voisine sur l'arête commune.
 */
export function ChoiceCards<T extends string>({
  value,
  onChange,
  options,
  label,
  className,
}: {
  value: T
  onChange: (v: T) => void
  options: Choice<T>[]
  /** Nom du groupe pour les lecteurs d'écran. */
  label?: string
  className?: string
}) {
  const move = (dir: 1 | -1) => {
    const selectable = options.filter((o) => !o.disabledReason)
    if (selectable.length === 0) return
    const i = selectable.findIndex((o) => o.value === value)
    const next = selectable[(i + dir + selectable.length) % selectable.length]
    onChange(next.value)
  }

  return (
    <div
      role='radiogroup'
      aria-label={label}
      className={cn('grid max-w-lg', className ?? 'grid-cols-2')}
      onKeyDown={(e) => {
        if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
          e.preventDefault()
          move(1)
        }
        if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
          e.preventDefault()
          move(-1)
        }
      }}
    >
      {options.map((o, i) => {
        const selected = o.value === value
        const first = i === 0
        const last = i === options.length - 1
        const blocked = Boolean(o.disabledReason)

        return (
          <button
            key={o.value}
            type='button'
            role='radio'
            aria-checked={selected}
            aria-disabled={blocked}
            title={o.disabledReason}
            /* Roving tabindex : le groupe est une seule étape de tabulation,
               on circule ensuite entre les cartes aux flèches. */
            tabIndex={selected ? 0 : -1}
            onClick={() => !blocked && onChange(o.value)}
            className={cn(
              'relative flex flex-col gap-1 border border-fk-line p-3 text-left transition-colors',
              'focus-visible:z-20 focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-fk-primary/30',
              first && 'rounded-l-lg',
              last && 'rounded-r-lg',
              !first && '-ml-px',
              selected
                ? 'z-10 border-fk-primary-border bg-fk-primary-soft'
                : 'bg-white hover:bg-neutral-50',
              blocked ? 'cursor-not-allowed opacity-55' : 'cursor-pointer'
            )}
          >
            <ChoiceCardHead option={o} selected={selected} />
          </button>
        )
      })}
    </div>
  )
}

/**
 * Variante multi-sélection de {@link ChoiceCards}, avec ordre.
 *
 * Même géométrie — cartes jointes, bordure partagée — mais les choix se
 * cumulent et leur rang compte : la carte retenue affiche sa position.
 * FK-39 : quand le domaine impose une liste non vide, décocher le dernier
 * élément est refusé avec son infobulle plutôt que de produire un état que
 * l'API rejetterait après coup.
 */
export function OrderedChoiceCards<T extends string>({
  value,
  onChange,
  options,
  label,
  className,
  minSelectedReason = 'At least one entry is required.',
}: {
  /** Liste ordonnée : l'ordre est la précédence. */
  value: T[]
  onChange: (v: T[]) => void
  options: Choice<T>[]
  label?: string
  className?: string
  minSelectedReason?: string
}) {
  const toggle = (v: T) => {
    if (value.includes(v)) {
      /* Le dernier ne se retire pas : la liste doit rester non vide. */
      if (value.length === 1) return
      onChange(value.filter((x) => x !== v))
    } else {
      onChange([...value, v])
    }
  }

  return (
    <div
      role='group'
      aria-label={label}
      className={cn('grid max-w-lg', className ?? 'grid-cols-2')}
    >
      {options.map((o, i) => {
        const selected = value.includes(o.value)
        const pinned = selected && value.length === 1
        const first = i === 0
        const last = i === options.length - 1

        return (
          <button
            key={o.value}
            type='button'
            role='checkbox'
            aria-checked={selected}
            aria-disabled={pinned}
            title={pinned ? minSelectedReason : undefined}
            onClick={() => toggle(o.value)}
            className={cn(
              'relative flex flex-col gap-1 border border-fk-line p-3 text-left transition-colors',
              'focus-visible:z-20 focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-fk-primary/30',
              first && 'rounded-l-lg',
              last && 'rounded-r-lg',
              !first && '-ml-px',
              selected
                ? 'z-10 border-fk-primary-border bg-fk-primary-soft'
                : 'bg-white hover:bg-neutral-50',
              pinned ? 'cursor-not-allowed' : 'cursor-pointer'
            )}
          >
            <ChoiceCardHead
              option={o}
              selected={selected}
              rank={selected ? value.indexOf(o.value) + 1 : undefined}
            />
          </button>
        )
      })}
    </div>
  )
}

/** Tête de carte partagée par les deux variantes — icône, libellé, coche. */
function ChoiceCardHead<T extends string>({
  option,
  selected,
  rank,
}: {
  option: Choice<T>
  selected: boolean
  rank?: number
}) {
  return (
    <>
      <span className='flex items-center gap-2'>
        {option.icon && (
          <option.icon
            className={cn(
              'size-3.5 shrink-0',
              selected ? 'text-fk-primary-text' : 'text-neutral-400'
            )}
            strokeWidth={1.75}
          />
        )}
        <span
          className={cn(
            'text-xs font-medium',
            selected ? 'text-fk-primary-text' : 'text-neutral-900'
          )}
        >
          {option.label}
        </span>
        <span className='flex-1' />
        <span
          className={cn(
            'tnum grid size-4 shrink-0 place-items-center rounded-full border text-[10px] font-semibold transition-colors',
            selected
              ? 'border-fk-primary bg-fk-primary text-white'
              : 'border-fk-line bg-white'
          )}
        >
          {selected &&
            (rank !== undefined ? rank : <Check className='size-2.5' strokeWidth={3} />)}
        </span>
      </span>
      {option.description && (
        <span
          className={cn(
            'text-xs leading-relaxed',
            selected ? 'text-fk-primary-text/80' : 'text-neutral-500'
          )}
        >
          {option.description}
        </span>
      )}
    </>
  )
}

/* -------------------------------------------------------------- */
/* ChipInput — champ multi-valeurs libres                           */
/* -------------------------------------------------------------- */

/**
 * Remplace les boutons « Add … » pleine largeur.
 *
 * Réservé aux valeurs **libres** — URI de redirection, origines web, en-têtes.
 * Dès que la valeur est une clé étrangère, c'est {@link EntityPicker} qu'il
 * faut (FK-28) : ici rien n'empêche de saisir un nom qui n'existe pas.
 */
export function ChipInput({
  values,
  onChange,
  placeholder,
  emptyHint,
  disabled,
}: {
  values: string[]
  onChange: (next: string[]) => void
  placeholder: string
  emptyHint?: string
  disabled?: boolean
}) {
  const [draft, setDraft] = useState('')

  const commit = () => {
    const v = draft.trim()
    if (!v || values.includes(v)) return
    onChange([...values, v])
    setDraft('')
  }

  return (
    <div className='space-y-2'>
      {values.length > 0 ? (
        <ul className='flex flex-wrap gap-1.5'>
          {values.map((v) => (
            <li
              key={v}
              className='inline-flex items-center gap-1.5 rounded border border-fk-line bg-neutral-50 py-1 pl-2 pr-1 font-mono-ui text-xs text-neutral-700'
            >
              {v}
              <button
                type='button'
                aria-label={`Remove ${v}`}
                disabled={disabled}
                onClick={() => onChange(values.filter((x) => x !== v))}
                className='grid size-4 cursor-pointer place-items-center rounded text-neutral-400 hover:bg-neutral-200 hover:text-neutral-700 disabled:cursor-not-allowed'
              >
                <X className='size-3' />
              </button>
            </li>
          ))}
        </ul>
      ) : (
        /* FK-32 : l'absence est nommée, jamais laissée vide. */
        emptyHint && <p className='text-xs text-neutral-400'>{emptyHint}</p>
      )}
      <div className='flex max-w-lg gap-2'>
        <Input
          value={draft}
          disabled={disabled}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault()
              commit()
            }
          }}
          placeholder={placeholder}
          className='font-mono-ui text-xs'
        />
        <Button
          type='button'
          variant='outline'
          size='icon'
          disabled={disabled}
          onClick={commit}
          aria-label='Add'
        >
          <Plus className='size-4' />
        </Button>
      </div>
    </div>
  )
}
