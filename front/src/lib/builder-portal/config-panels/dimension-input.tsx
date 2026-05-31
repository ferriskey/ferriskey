import { ChevronDown } from 'lucide-react'
import { useMemo, type ReactNode } from 'react'

interface DimensionInputProps {
  label?: string
  value: string | undefined
  onChange: (value: string) => void
  /**
   * Override the default unit list. Useful for places where some units make
   * no sense — `fr` is only valid as a `grid-template-*` column/row size,
   * for instance.
   */
  units?: Unit[]
  /**
   * When provided, wraps the inputs in the shared dense row layout from
   * `shared-fields.tsx`. Omit when embedding the dimension input as part of
   * a larger composite (e.g., inside `LinkedSidesInput`'s side editors).
   */
  rowed?: boolean
}

type Unit = 'px' | '%' | 'rem' | 'em' | 'vw' | 'vh' | 'fr' | 'auto' | 'raw'

const DEFAULT_UNITS: Unit[] = ['px', '%', 'rem', 'em', 'vw', 'vh', 'auto', 'raw']

const UNIT_LABEL: Record<Unit, string> = {
  px: 'px',
  '%': '%',
  rem: 'rem',
  em: 'em',
  vw: 'vw',
  vh: 'vh',
  fr: 'fr',
  auto: 'auto',
  raw: 'css',
}

interface Parsed {
  num: string
  unit: Unit
  /**
   * The value couldn't be split into number + unit (e.g., it's a `calc()`
   * expression). We surface a `raw` editor in that case so the admin still
   * has full access — but the number input is hidden.
   */
  raw: boolean
}

/**
 * Split a CSS value into `[number, unit]`. Recognises:
 *  - `""` (empty)            → number empty, default `px`
 *  - `"auto"`                → unit `auto`
 *  - `"16px"`, `"100%"`, …   → split with regex
 *  - everything else         → falls back to `raw` text mode so values like
 *                              `calc(100% - 16px)` aren't silently zeroed
 *                              when the admin opens an existing card.
 */
function parse(raw: string | undefined): Parsed {
  if (raw === undefined || raw === '') return { num: '', unit: 'px', raw: false }
  if (raw.trim() === 'auto') return { num: '', unit: 'auto', raw: false }
  const m = raw.match(/^\s*(-?\d*\.?\d+)\s*(px|%|rem|em|vw|vh|fr)?\s*$/)
  if (m) return { num: m[1], unit: ((m[2] as Unit) ?? 'px') as Unit, raw: false }
  return { num: raw, unit: 'raw', raw: true }
}

function format(num: string, unit: Unit): string {
  if (unit === 'auto') return 'auto'
  if (unit === 'raw') return num
  if (num === '' || num === '-') return ''
  return `${num}${unit}`
}

/**
 * Number-with-unit field used everywhere a CSS length is stored as a
 * string in node props (width, height, padding, gap, …). Replaces the raw
 * `TextField` so admins type the number and pick the unit from a dropdown
 * — far faster than retyping `"16px"` and immune to syntax typos
 * (`"16 px"`, `"16ppx"`).
 */
export function DimensionInput({ label, value, onChange, units, rowed = true }: DimensionInputProps) {
  const parsed = useMemo(() => parse(value), [value])
  const allowedUnits = units ?? DEFAULT_UNITS

  // `auto` disables the number field entirely — there's no number to put in
  // front of it. `raw` swaps the number input for a fluid text input so the
  // admin can edit `calc(...)`-style expressions in place.
  const isAuto = parsed.unit === 'auto'
  const isRaw = parsed.unit === 'raw'

  const handleNumChange = (next: string) => {
    onChange(format(next, parsed.unit))
  }

  const handleUnitChange = (nextUnit: Unit) => {
    if (nextUnit === 'auto') {
      onChange('auto')
      return
    }
    if (nextUnit === 'raw') {
      onChange(parsed.num || '')
      return
    }
    onChange(format(parsed.num, nextUnit))
  }

  const body = (
    <div className='flex w-full items-center gap-1'>
      {isRaw ? (
        <input
          type='text'
          className='h-7 w-full rounded border border-border bg-background px-2 font-mono text-[10px]'
          value={parsed.num}
          onChange={(e) => onChange(e.target.value)}
          placeholder='calc(…)'
        />
      ) : (
        <input
          type='number'
          className='h-7 w-full min-w-0 rounded border border-border bg-background px-2 text-right text-[11px] tabular-nums disabled:opacity-50'
          value={isAuto ? '' : parsed.num}
          disabled={isAuto}
          onChange={(e) => handleNumChange(e.target.value)}
        />
      )}
      <UnitMenu
        value={parsed.unit}
        units={allowedUnits}
        onChange={handleUnitChange}
      />
    </div>
  )

  if (!rowed) return body
  return <FieldRow label={label ?? ''}>{body}</FieldRow>
}

/**
 * Tiny dropdown rendered next to the number input. Uses a native `<select>`
 * to keep the keyboard story sane and the bundle small — a custom popover
 * would be prettier but this stays consistent with other native pickers in
 * the builder.
 */
function UnitMenu({
  value,
  units,
  onChange,
}: {
  value: Unit
  units: Unit[]
  onChange: (next: Unit) => void
}) {
  return (
    <div className='relative shrink-0'>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value as Unit)}
        className='h-7 cursor-pointer appearance-none rounded border border-border bg-background pl-1.5 pr-5 text-[10px] uppercase'
      >
        {units.map((u) => (
          <option key={u} value={u}>
            {UNIT_LABEL[u]}
          </option>
        ))}
      </select>
      <ChevronDown
        className='pointer-events-none absolute right-1 top-1/2 -translate-y-1/2 text-muted-foreground'
        size={10}
      />
    </div>
  )
}

/**
 * Local copy of the dense row from `shared-fields.tsx` so this file doesn't
 * pull in a circular dep — the layout is intentionally identical so a row
 * built from `<DimensionInput rowed>` lines up pixel-for-pixel with a row
 * built from `<TextField>` / `<ColorField>` in the same panel.
 */
function FieldRow({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className='flex min-h-7 items-center gap-2'>
      <span className='w-[42%] shrink-0 truncate text-xs text-muted-foreground'>
        {label}
      </span>
      <div className='flex min-w-0 flex-1 items-center'>{children}</div>
    </div>
  )
}
