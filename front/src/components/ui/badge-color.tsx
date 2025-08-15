import clsx from 'clsx'
import { PropsWithChildren } from 'react'
import { BadgeColorScheme } from './badge-color.enum'



export interface BadgeColorProps {
  color: BadgeColorScheme
  className?: string
}

const BadgeColor = ({ children, color, className }: PropsWithChildren<BadgeColorProps>) => {
  const colorStyles = {
    [BadgeColorScheme.PRIMARY]: 'bg-primary/10 text-primary ring-primary/20',
    [BadgeColorScheme.GRAY]: 'bg-gray-50 text-gray-600 ring-gray-500/10',
    [BadgeColorScheme.RED]: 'bg-red-50 text-red-700 ring-red-600/10',
    [BadgeColorScheme.YELLOW]:
      'bg-yellow-50 text-yellow-800 ring-yellow-600/20',
    [BadgeColorScheme.GREEN]: 'bg-green-50 text-green-700 ring-green-600/20',
    [BadgeColorScheme.BLUE]: 'bg-blue-50 text-blue-700 ring-blue-700/10',
    [BadgeColorScheme.INDIGO]:
      'bg-indigo-50 text-indigo-700 ring-indigo-700/10',
    [BadgeColorScheme.PURPLE]:
      'bg-purple-50 text-purple-700 ring-purple-700/10',
    [BadgeColorScheme.PINK]: 'bg-pink-50 text-pink-700 ring-pink-700/10',
    [BadgeColorScheme.FLUID]:
      'bg-opacity-10 bg-[#3daeb8] text-[#2c7d85] ring-[#3daeb8]/20 rounded-full',
  }

  return (
    <span
      className={clsx('inline-flex items-center rounded px-2 py-1 text-xs font-medium ring-1 ring-inset', colorStyles[color], className)}
    >
      {children}
    </span>
  )
}

export default BadgeColor
