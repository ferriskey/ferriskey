interface EntityAvatarProps {
  label: string
  color?: string
  size?: 'sm' | 'md'
}

const SIZE = {
  sm: 'h-8 w-8 text-sm',
  md: 'h-10 w-10 text-base',
}

export function EntityAvatar({ label, color = '#F97316', size = 'md' }: EntityAvatarProps) {
  return (
    <div
      className={`${SIZE[size]} rounded-md flex items-center justify-center shrink-0 font-bold text-white`}
      style={{ backgroundColor: color }}
    >
      {label?.[0]?.toUpperCase() || '?'}
    </div>
  )
}
