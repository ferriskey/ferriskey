import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { FieldRow, Section } from '@/components/kit'
import type { Schemas } from '@/api/api.client'

export const NO_LAYOUT = '__none__'

export interface ThemeLayoutTabProps {
  layouts: Schemas.PortalLayout[]
  layoutId: string
  savedLayoutId: string
  onLayoutChange: (value: string) => void
}

export default function ThemeLayoutTab({
  layouts,
  layoutId,
  savedLayoutId,
  onLayoutChange,
}: ThemeLayoutTabProps) {
  const selected = layouts.find((l) => l.id === layoutId)
  const detaching = layouts.find((l) => l.id === savedLayoutId && savedLayoutId !== layoutId)

  return (
    <Section
      title='Layout'
      description='The header, footer and card wrapper every page of this theme is rendered inside.'
    >
      <FieldRow
        label='Portal layout'
        description='Optional. Without a layout the portal renders the page tree on its own.'
      >
        <div className='max-w-sm'>
          <Select value={layoutId} onValueChange={onLayoutChange}>
            <SelectTrigger className='w-full'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={NO_LAYOUT}>No layout — render the page bare</SelectItem>
              {layouts.map((layout) => (
                <SelectItem key={layout.id} value={layout.id}>
                  {layout.name}
                  {layout.is_default ? ' · default' : ''}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          {selected && (
            <p className='mt-1.5 text-xs text-neutral-400'>
              While this theme uses it, “{selected.name}” cannot be deleted.
            </p>
          )}
          {detaching && (
            <p className='mt-1.5 text-xs text-fk-amber'>
              Saving detaches “{detaching.name}” from this theme; it becomes deletable
              unless another theme holds it.
            </p>
          )}
        </div>
      </FieldRow>
    </Section>
  )
}
