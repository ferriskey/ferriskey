import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { cn } from '@/lib/utils'
import { ChevronRight } from 'lucide-react'
import { MapperTemplate } from '../../constants/protocol-mapper-templates'

// ─── Quick Start card ─────────────────────────────────────────────────────────

function QuickStartCard({
  template,
  onClick,
}: {
  template: MapperTemplate
  onClick: () => void
}) {
  return (
    <button
      type='button'
      onClick={onClick}
      className={cn(
        'flex flex-col gap-1.5 rounded-lg border border-border bg-card p-4 text-left',
        'transition-all hover:border-primary/60 hover:bg-muted/40 hover:shadow-sm',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring'
      )}
    >
      <span className='text-2xl leading-none'>{template.icon}</span>
      <span className='text-sm font-semibold'>{template.name}</span>
      <span className='text-xs text-muted-foreground leading-snug'>{template.description}</span>
    </button>
  )
}

// ─── Catalog list row ─────────────────────────────────────────────────────────

function CatalogRow({
  template,
  onClick,
}: {
  template: MapperTemplate
  onClick: () => void
}) {
  return (
    <button
      type='button'
      onClick={onClick}
      className='group flex w-full items-center gap-3 rounded-md px-3 py-2.5 text-left transition-colors hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring'
    >
      <span className='shrink-0 text-xl leading-none'>{template.icon}</span>
      <div className='min-w-0 flex-1'>
        <p className='text-sm font-medium'>{template.name}</p>
        <p className='truncate text-xs text-muted-foreground'>{template.description}</p>
        {!template.isCustom && (
          <p className='mt-0.5 truncate font-mono text-xs text-muted-foreground/50'>
            {template.mapper_type}
          </p>
        )}
      </div>
      <ChevronRight className='h-4 w-4 shrink-0 text-muted-foreground/30 transition-colors group-hover:text-muted-foreground' />
    </button>
  )
}

// ─── Section header ───────────────────────────────────────────────────────────

function SectionHeader({ title, subtitle }: { title: string; subtitle: string }) {
  return (
    <div className='mb-3'>
      <p className='text-sm font-semibold'>{title}</p>
      <p className='text-xs text-muted-foreground'>{subtitle}</p>
    </div>
  )
}

// ─── Modal ────────────────────────────────────────────────────────────────────

interface MapperTemplatePickerModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  quickStartTemplates: MapperTemplate[]
  catalogTemplates: MapperTemplate[]
  onSelectTemplate: (template: MapperTemplate) => void
}

export default function MapperTemplatePickerModal({
  open,
  onOpenChange,
  quickStartTemplates,
  catalogTemplates,
  onSelectTemplate,
}: MapperTemplatePickerModalProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className='sm:max-w-xl max-h-[85vh] flex flex-col'>
        <DialogHeader>
          <DialogTitle>Add Protocol Mapper</DialogTitle>
        </DialogHeader>

        <DialogBody className='overflow-y-auto flex flex-col gap-6 pr-1'>
          {/* Quick Start */}
          <section>
            <SectionHeader
              title='Quick Start'
              subtitle='Pre-configured mappers for common use cases — just add a name and go.'
            />
            <div className='grid grid-cols-2 gap-2.5 sm:grid-cols-3'>
              {quickStartTemplates.map((template) => (
                <QuickStartCard
                  key={template.id}
                  template={template}
                  onClick={() => onSelectTemplate(template)}
                />
              ))}
            </div>
          </section>

          {/* By Configuration */}
          <section>
            <SectionHeader
              title='By configuration'
              subtitle='Pick any mapper type and configure every setting yourself.'
            />
            <div className='rounded-lg border border-border divide-y divide-border overflow-hidden'>
              {catalogTemplates.map((template) => (
                <CatalogRow
                  key={template.id}
                  template={template}
                  onClick={() => onSelectTemplate(template)}
                />
              ))}
            </div>
          </section>
        </DialogBody>
      </DialogContent>
    </Dialog>
  )
}
