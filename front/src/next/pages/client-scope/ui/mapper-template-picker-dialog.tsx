import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import {
  MAPPER_CATALOG,
  QUICK_START_TEMPLATES,
  type MapperTemplate,
} from '@/pages/client-scope/constants/protocol-mapper-templates'

export interface MapperTemplatePickerDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSelect: (template: MapperTemplate) => void
}

function TemplateGroup({
  label,
  hint,
  templates,
  onSelect,
}: {
  label: string
  hint: string
  templates: MapperTemplate[]
  onSelect: (template: MapperTemplate) => void
}) {
  return (
    <div>
      <p className='text-[11px] font-medium uppercase tracking-wide text-neutral-400'>{label}</p>
      <p className='pb-2 text-xs text-neutral-500'>{hint}</p>
      <div className='space-y-1.5'>
        {templates.map((template) => (
          <button
            key={template.id}
            type='button'
            onClick={() => onSelect(template)}
            className={cn(
              tokens.surface.panel,
              'flex w-full cursor-pointer items-start gap-3 p-3 text-left transition-colors',
              'hover:border-fk-primary-border hover:bg-fk-primary-soft/30',
              'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-fk-primary/30'
            )}
          >
            <span className='text-base leading-none'>{template.icon}</span>
            <span className='min-w-0 flex-1'>
              <span className='block text-xs font-medium text-neutral-900'>{template.name}</span>
              <span className='mt-0.5 block text-xs text-neutral-500'>{template.description}</span>
              {!template.isCustom && (
                <span className='mt-0.5 block truncate font-mono-ui text-[11px] text-neutral-400'>
                  {template.mapper_type}
                </span>
              )}
            </span>
          </button>
        ))}
      </div>
    </div>
  )
}

export default function MapperTemplatePickerDialog({
  open,
  onOpenChange,
  onSelect,
}: MapperTemplatePickerDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className='flex max-h-[85vh] flex-col sm:max-w-xl'>
        <DialogHeader>
          <DialogTitle>Add a protocol mapper</DialogTitle>
          <DialogDescription>
            The template fills in the mapper type and the fields it expects. Custom mapper
            leaves everything to type.
          </DialogDescription>
        </DialogHeader>

        <DialogBody className='scrollbar-hide flex flex-col gap-5 overflow-y-auto pr-1'>
          <TemplateGroup
            label='Quick start'
            hint='Pre-configured mappers for the usual claims — name it and go.'
            templates={QUICK_START_TEMPLATES}
            onSelect={onSelect}
          />
          <TemplateGroup
            label='By configuration'
            hint='Pick any mapper type and set every field yourself.'
            templates={MAPPER_CATALOG}
            onSelect={onSelect}
          />
        </DialogBody>
      </DialogContent>
    </Dialog>
  )
}
