import ProviderIcon from '@/pages/identity-providers/components/provider-icon'
import { cn } from '@/lib/utils'
import { providerLogo } from '../provider-status'

export interface ProviderTileProps {
  providerId: string
  size?: 'sm' | 'md'
  className?: string
}

export default function ProviderTile({
  providerId,
  size = 'sm',
  className,
}: ProviderTileProps) {
  return (
    <span
      className={cn(
        'grid shrink-0 place-items-center rounded-md border border-fk-line bg-white',
        size === 'sm' ? 'size-9' : 'size-15',
        className
      )}
    >
      <ProviderIcon icon={providerLogo(providerId)} size={size === 'sm' ? 'sm' : 'md'} />
    </span>
  )
}
