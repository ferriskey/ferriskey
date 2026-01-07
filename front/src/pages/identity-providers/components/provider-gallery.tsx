import { useState, useMemo } from 'react'
import { Search } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import {
  PROVIDER_TEMPLATES,
  CUSTOM_PROVIDER_TEMPLATE,
  type ProviderTemplate,
} from '@/constants/identity-provider-templates'
import ProviderIcon from './provider-icon'

interface ProviderGalleryProps {
  onSelect: (template: ProviderTemplate) => void
  selectedId?: string
}

const categoryLabels: Record<ProviderTemplate['category'], string> = {
  social: 'Social',
  enterprise: 'Enterprise',
  developer: 'Developer Tools',
  custom: 'Custom',
}

const categoryOrder: ProviderTemplate['category'][] = ['social', 'enterprise', 'developer', 'custom']

export default function ProviderGallery({ onSelect, selectedId }: ProviderGalleryProps) {
  const [search, setSearch] = useState('')

  const filteredTemplates = useMemo(() => {
    if (!search) return PROVIDER_TEMPLATES

    const searchLower = search.toLowerCase()
    return PROVIDER_TEMPLATES.filter(
      (t) =>
        t.displayName.toLowerCase().includes(searchLower) ||
        t.description.toLowerCase().includes(searchLower)
    )
  }, [search])

  const groupedTemplates = useMemo(() => {
    const groups: Record<ProviderTemplate['category'], ProviderTemplate[]> = {
      social: [],
      enterprise: [],
      developer: [],
      custom: [],
    }

    filteredTemplates.forEach((t) => {
      groups[t.category].push(t)
    })

    return groups
  }, [filteredTemplates])

  const hasResults = filteredTemplates.length > 0

  return (
    <div className='space-y-4'>
      {/* Search */}
      <div className='relative max-w-md'>
        <Search className='absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground' />
        <Input
          type='search'
          placeholder='Search providers...'
          className='pl-10'
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>

      {!hasResults && (
        <div className='text-center py-8'>
          <p className='text-muted-foreground'>No providers found matching '{search}'</p>
        </div>
      )}

      {/* Provider Categories */}
      {categoryOrder.map((category) => {
        const templates = groupedTemplates[category]
        if (templates.length === 0 && category !== 'custom') return null

        return (
          <div key={category} className='space-y-3'>
            <h3 className='text-sm font-medium text-muted-foreground'>
              {categoryLabels[category]}
            </h3>

            <div className='grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3'>
              {templates.map((template) => (
                <ProviderCard
                  key={template.id}
                  template={template}
                  isSelected={selectedId === template.id}
                  onClick={() => onSelect(template)}
                />
              ))}

              {/* Custom provider card at the end */}
              {category === 'custom' && (
                <ProviderCard
                  template={CUSTOM_PROVIDER_TEMPLATE}
                  isSelected={selectedId === CUSTOM_PROVIDER_TEMPLATE.id}
                  onClick={() => onSelect(CUSTOM_PROVIDER_TEMPLATE)}
                />
              )}
            </div>
          </div>
        )
      })}
    </div>
  )
}

interface ProviderCardProps {
  template: ProviderTemplate
  isSelected: boolean
  onClick: () => void
}

function ProviderCard({ template, isSelected, onClick }: ProviderCardProps) {
  return (
    <Card
      className={cn(
        'cursor-pointer transition-all duration-200 hover:shadow-md hover:scale-[1.02] hover:border-primary/50',
        isSelected && 'ring-2 ring-primary border-primary'
      )}
      onClick={onClick}
    >
      <CardContent className='p-3 flex flex-col items-center text-center gap-2'>
        <div className='h-10 w-10 flex items-center justify-center'>
          <ProviderIcon icon={template.icon} size='md' />
        </div>

        <div className='space-y-0.5'>
          <h4 className='font-medium text-sm leading-tight'>{template.displayName}</h4>
          <p className='text-xs text-muted-foreground line-clamp-1'>
            {template.description}
          </p>
        </div>

        <Badge variant='outline' className='text-[10px] px-1.5 py-0'>
          {template.provider_type.toUpperCase()}
        </Badge>
      </CardContent>
    </Card>
  )
}
