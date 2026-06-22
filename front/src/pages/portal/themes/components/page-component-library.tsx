import { useDraggable } from '@dnd-kit/core'
import { ChevronDown, Sparkles } from 'lucide-react'
import { Fragment, useState } from 'react'
import {
  HIDDEN_BLOCKS_BY_PAGE_TYPE,
  LAYOUT_ONLY_BLOCK_TYPES,
  PORTAL_PRESETS,
  REQUIRED_BLOCK_TYPES,
  RESTRICTED_TO_PAGE_TYPE,
  portalComponents,
  type PortalPreset,
} from '@/lib/builder-portal'
import { ComponentTree, useBuilderContext, type ComponentDefinition } from '@/lib/builder-core'
import type { Schemas } from '@/api/api.client'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { cn } from '@/lib/utils'
import { SidebarTabs, type SidebarTab } from '@/pages/portal-layouts/components/sidebar-tabs'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'

interface Props {
  /** Block types the API requires for the currently-edited page. */
  requiredTypes: string[]
  /**
   * Drives the per-page exclusion list (see `HIDDEN_BLOCKS_BY_PAGE_TYPE`).
   * Hides blocks that don't make semantic sense for the current page — e.g.,
   * no "Forgot password" link on the Register page.
   */
  pageType?: Schemas.PortalPageType
}

/**
 * Map every portal block type to a coarse category so the palette can be
 * grouped instead of presented as a flat list of 15 items. The required
 * blocks (those returned by the API for the current page) keep their own
 * "Required for this page" group, which short-circuits this mapping.
 */
const COMPONENT_GROUPS: Array<{ id: string; label: string; types: string[] }> = [
  { id: 'layout', label: 'Layout', types: ['container', 'div', 'card'] },
  {
    id: 'content',
    label: 'Content',
    types: ['heading', 'text', 'image', 'spacer', 'divider', 'form_error_banner'],
  },
  {
    id: 'form',
    label: 'Form & Actions',
    types: [
      'input',
      'button',
      'magic_link_button',
      'passkey_button',
      'forgot_password_link',
      'back_to_login_link',
      'register_link',
    ],
  },
  {
    id: 'identity',
    label: 'Identity fields',
    types: ['first_name_input', 'last_name_input', 'username_input'],
  },
]

export function PageComponentLibrary({ requiredTypes, pageType }: Props) {
  const [tab, setTab] = useState<SidebarTab>('components')
  const hidden = (pageType && HIDDEN_BLOCKS_BY_PAGE_TYPE[pageType]) ?? null
  const isHidden = (type: string) => (hidden ? hidden.has(type) : false)
  // A block restricted to a set of page types is hidden when the current
  // page isn't in that set (e.g., `totp_qr_code` only on `totp_setup`).
  const isRestricted = (type: string) => {
    const allowed = RESTRICTED_TO_PAGE_TYPE[type]
    return allowed ? !pageType || !allowed.has(pageType) : false
  }
  const generic = portalComponents.filter(
    (c) =>
      !REQUIRED_BLOCK_TYPES.has(c.type) &&
      !LAYOUT_ONLY_BLOCK_TYPES.has(c.type) &&
      !isHidden(c.type) &&
      !isRestricted(c.type)
  )
  const genericByType = new Map(generic.map((c) => [c.type, c]))
  const required = requiredTypes
    .filter((t) => !isHidden(t))
    .map((type) => portalComponents.find((c) => c.type === type))
    .filter((c): c is ComponentDefinition => Boolean(c))

  // Items declared in `COMPONENT_GROUPS.types` that actually exist in the
  // adapter — defends against typos and against blocks being removed from
  // the registry without updating the groups table.
  const groupedItems = COMPONENT_GROUPS.map((g) => ({
    ...g,
    items: g.types
      .map((t) => genericByType.get(t))
      .filter((c): c is ComponentDefinition => Boolean(c)),
  }))

  // Any generic block whose type isn't listed in a group ends up in
  // "Other" — surfaces oversights instead of silently hiding new blocks.
  const groupedTypes = new Set(COMPONENT_GROUPS.flatMap((g) => g.types))
  const ungrouped = generic.filter((c) => !groupedTypes.has(c.type))

  return (
    <div className='flex w-full min-w-0 flex-col'>
      <SidebarTabs current={tab} onChange={setTab} />
      {tab === 'components' && (
        <div className='flex w-full min-w-0 flex-col gap-3 p-2'>
          {groupedItems.map((g) =>
            g.items.length === 0 ? null : (
              <Group key={g.id} title={g.label}>
                {g.items.map((def) => (
                  <DraggableComponent key={def.type} definition={def} />
                ))}
              </Group>
            )
          )}
          {ungrouped.length > 0 && (
            <Group title='Other'>
              {ungrouped.map((def) => (
                <DraggableComponent key={def.type} definition={def} />
              ))}
            </Group>
          )}
          {required.length > 0 && (
            <Group title='Required for this page' defaultOpen>
              {required.map((def) => (
                <DraggableComponent key={def.type} definition={def} />
              ))}
            </Group>
          )}
        </div>
      )}
      {tab === 'presets' && <PresetsTab />}
      {tab === 'tree' && <ComponentTree />}
    </div>
  )
}

/**
 * Collapsible group inside the components palette. Mirrors the theme
 * panel's `PanelSection` (chevron right, persisted open/closed state) so
 * both sides of the builder share the same affordance.
 */
function Group({
  title,
  defaultOpen = true,
  children,
}: {
  title: string
  defaultOpen?: boolean
  children: React.ReactNode
}) {
  const [open, setOpen] = useState(defaultOpen)
  return (
    <Collapsible open={open} onOpenChange={setOpen} className='flex flex-col gap-1.5'>
      <CollapsibleTrigger className='flex items-center gap-1 px-1 text-left text-[11px] font-semibold uppercase tracking-wider text-muted-foreground hover:text-foreground'>
        <ChevronDown
          className={cn('h-3 w-3 shrink-0 transition-transform', !open && '-rotate-90')}
        />
        <span>{title}</span>
      </CollapsibleTrigger>
      <CollapsibleContent>
        <div className='flex flex-col gap-1'>{children}</div>
      </CollapsibleContent>
    </Collapsible>
  )
}

function DraggableComponent({ definition }: { definition: ComponentDefinition }) {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id: `library-${definition.type}`,
    data: {
      source: 'library',
      type: definition.type,
    },
  })

  return (
    <div
      ref={setNodeRef}
      {...listeners}
      {...attributes}
      className={`flex w-full min-w-0 cursor-grab items-center gap-2 rounded-md border border-border bg-card p-2 text-sm transition-colors hover:bg-accent ${
        isDragging ? 'opacity-50' : ''
      }`}
    >
      {definition.icon && (
        <span className='flex h-5 w-5 shrink-0 items-center justify-center text-muted-foreground'>
          {definition.icon}
        </span>
      )}
      <span className='min-w-0 truncate'>{definition.label}</span>
    </div>
  )
}

/**
 * Presets tab: click-to-insert ready-made block trees. We don't make these
 * draggable — a preset is a tree with its own internal layout, dropping it
 * on a random sortable target would produce nonsense (e.g., dropping the
 * sign-in card inside another card's footer). Click appends at the canvas
 * root, which is always the safe insertion point.
 */
function PresetsTab() {
  const { tree, setTree } = useBuilderContext()
  const [selectedPreset, setSelectedPreset] = useState<PortalPreset | null>(null)
  const [open, setOpen] = useState(false)

  function handleReplace(preset: PortalPreset) {
    // alert modale submit
    if (tree.length) {
      setSelectedPreset(preset)
      setOpen(true)
    } else {
      setTree([...preset.factory()])
    }
  }

  function handleSubmit() {
    setTree([...selectedPreset!.factory()])
    setOpen(false)
    setSelectedPreset(null)
  }

  function handleCancel() {
    setOpen(false)
    setSelectedPreset(null)
  }

  return (
    <Fragment>
      <ConfirmDeleteAlert
        open={open}
        title='Confirm actual preset deletion'
        description='This action cannot be undone.'
        confirmText='confirm'
        onConfirm={handleSubmit}
        onCancel={handleCancel}
      />
      <div className='flex flex-col gap-2 p-2'>
        <div className='flex items-center gap-1.5 px-1 pb-1 text-[11px] text-muted-foreground'>
          <Sparkles size={12} />
          <span>Click a preset to append it to the page.</span>
        </div>
        {PORTAL_PRESETS.map((p) => (
          <button
            key={p.id}
            type='button'
            onClick={() => handleReplace(p)}
            className='flex flex-col gap-0.5 rounded-md border border-border bg-card p-3 text-left text-xs transition-colors hover:border-primary hover:bg-accent'
          >
            <span className='font-medium text-foreground'>{p.label}</span>
            <span className='text-muted-foreground'>{p.description}</span>
          </button>
        ))}
      </div>
    </Fragment>
  )
}
