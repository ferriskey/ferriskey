import { Link, Unlink } from 'lucide-react'
import { useMemo, type ReactNode } from 'react'
import { DimensionInput } from './dimension-input'

interface LinkedSidesInputProps {
  label: string
  value: string | undefined
  onChange: (value: string) => void
  /**
   * Per-corner / per-side mode. `'sides'` (default) edits Top/Right/Bottom/Left
   * — used for `padding` and `margin`. `'corners'` edits the four corners in
   * `top-left / top-right / bottom-right / bottom-left` order — used for
   * `border-radius`.
   */
  mode?: 'sides' | 'corners'
}

interface Parsed {
  linked: boolean
  values: [string, string, string, string]
}

/**
 * Parse the CSS shorthand stored in node props. Accepts:
 *  - `""`                  → all empty (linked)
 *  - `"16px"`              → all 4 = 16px (linked)
 *  - `"16px 8px"`          → top/bottom = 16px, left/right = 8px (unlinked)
 *  - `"1px 2px 3px"`       → top = 1, left/right = 2, bottom = 3 (unlinked)
 *  - `"1px 2px 3px 4px"`   → top, right, bottom, left (unlinked)
 *
 * Anything else falls back to "linked, the whole string treated as one
 * value" so legacy values like `var(--x)` aren't silently wiped.
 */
function parse(raw: string | undefined): Parsed {
  if (!raw || raw.trim() === '') {
    return { linked: true, values: ['', '', '', ''] }
  }
  const parts = raw.trim().split(/\s+/)
  if (parts.length === 1) {
    return { linked: true, values: [parts[0], parts[0], parts[0], parts[0]] }
  }
  if (parts.length === 2) {
    const [tb, lr] = parts
    return { linked: false, values: [tb, lr, tb, lr] }
  }
  if (parts.length === 3) {
    const [t, lr, b] = parts
    return { linked: false, values: [t, lr, b, lr] }
  }
  if (parts.length === 4) {
    return { linked: false, values: [parts[0], parts[1], parts[2], parts[3]] }
  }
  // Unknown shape (e.g., CSS variable or function call) → keep as a single
  // linked value so the admin doesn't lose their work just by viewing it.
  return { linked: true, values: [raw, raw, raw, raw] }
}

/**
 * Serialise the 4 values back into a CSS shorthand, collapsing into the
 * shortest equivalent so common cases stay readable (`"16px"` rather than
 * `"16px 16px 16px 16px"`).
 */
function format(values: [string, string, string, string]): string {
  const [a, b, c, d] = values
  if (a === b && b === c && c === d) return a
  if (a === c && b === d) return `${a} ${b}`
  if (b === d) return `${a} ${b} ${c}`
  return `${a} ${b} ${c} ${d}`
}

/**
 * 4-sided / 4-cornered length editor. Default mode shows one input that
 * pilots every side at once; the link button toggles into a 4-input grid
 * with per-side editors. Matches the linked/unlinked pattern from Webflow
 * and Figma — common for padding/margin/border-radius.
 */
export function LinkedSidesInput({ label, value, onChange, mode = 'sides' }: LinkedSidesInputProps) {
  const parsed = useMemo(() => parse(value), [value])
  const linked = parsed.linked

  const setLinked = (linkedNext: boolean) => {
    if (linkedNext) {
      // Going back to linked mode collapses all four values to the first
      // one (the input shown in linked mode). Loses the per-side splits
      // but matches what the visible input in linked mode would write.
      onChange(parsed.values[0] ?? '')
    } else {
      // Going to unlinked mode keeps the current value across all 4 sides
      // (already true in linked state) — no string change needed.
      // Re-emit so a parent that's diffing on string changes still sees
      // the transition.
      onChange(format(parsed.values))
    }
  }

  const setIndex = (i: number, next: string) => {
    const arr = [...parsed.values] as [string, string, string, string]
    arr[i] = next
    onChange(format(arr))
  }

  const labels =
    mode === 'corners'
      ? (['Top-left', 'Top-right', 'Bottom-right', 'Bottom-left'] as const)
      : (['Top', 'Right', 'Bottom', 'Left'] as const)

  return (
    <div className='flex flex-col gap-1'>
      <div className='flex min-h-7 items-center gap-2'>
        <span className='w-[42%] shrink-0 truncate text-xs text-muted-foreground'>{label}</span>
        <div className='flex min-w-0 flex-1 items-center gap-1'>
          {linked ? (
            <DimensionInput
              value={parsed.values[0]}
              onChange={(v) => onChange(v)}
              rowed={false}
            />
          ) : (
            // Reserve the same horizontal space as the linked input so the
            // link/unlink button doesn't visibly jump.
            <span className='flex-1 text-[10px] text-muted-foreground'>
              {labels.length} sides
            </span>
          )}
          <button
            type='button'
            onClick={() => setLinked(!linked)}
            className='flex h-7 w-7 shrink-0 items-center justify-center rounded border border-border text-muted-foreground hover:bg-accent'
            title={linked ? 'Unlink sides' : 'Link sides'}
            aria-label={linked ? 'Unlink sides' : 'Link sides'}
          >
            {linked ? <Link size={12} /> : <Unlink size={12} />}
          </button>
        </div>
      </div>

      {!linked && (
        <div className='grid grid-cols-2 gap-1 pl-[42%]'>
          {labels.map((sideLabel, i) => (
            <SideEditor
              key={sideLabel}
              label={sideLabel}
              value={parsed.values[i]}
              onChange={(v) => setIndex(i, v)}
            />
          ))}
        </div>
      )}
    </div>
  )
}

function SideEditor({
  label,
  value,
  onChange,
}: {
  label: string
  value: string
  onChange: (v: string) => void
}): ReactNode {
  return (
    <div className='flex flex-col gap-0.5'>
      <span className='truncate text-[9px] uppercase tracking-wide text-muted-foreground'>
        {label}
      </span>
      <DimensionInput value={value} onChange={onChange} rowed={false} />
    </div>
  )
}
