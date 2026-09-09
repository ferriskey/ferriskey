import { AnimatePresence, motion, useReducedMotion } from 'motion/react'
import { ReactNode, useLayoutEffect, useRef, useState } from 'react'
import { cn } from '@/lib/utils'
import { Button } from './button'

export interface SaveBarAction {
  label: string
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link'
  onClick: () => void
  icon?: ReactNode
}

export interface SaveBarProps {
  show: boolean
  title: string
  description?: string
  onCancel?: () => void
  actions: SaveBarAction[]
  cancelLabel?: string
  className?: string
}

export default function SaveBar({
  show,
  title,
  description,
  actions,
  onCancel,
  cancelLabel = 'Discard',
  className,
}: SaveBarProps) {
  const prefersReducedMotion = useReducedMotion()
  const barRef = useRef<HTMLDivElement>(null)
  const [barHeight, setBarHeight] = useState(0)

  useLayoutEffect(() => {
    if (!show || !barRef.current) return
    const bar = barRef.current
    const update = () => setBarHeight(bar.getBoundingClientRect().height)
    const observer = new ResizeObserver(update)
    observer.observe(bar)
    return () => observer.disconnect()
  }, [show])

  return (
    <>
      <div
        aria-hidden='true'
        className='shrink-0 transition-[height] duration-200 ease-out motion-reduce:transition-none'
        style={{
          height: show && barHeight
            ? `calc(${barHeight}px + 3rem + env(safe-area-inset-bottom))`
            : 0,
        }}
      />

      <AnimatePresence>
        {show && (
          <motion.div
            ref={barRef}
            initial={prefersReducedMotion ? { opacity: 0 } : { y: 100, opacity: 0 }}
            animate={prefersReducedMotion ? { opacity: 1 } : { y: 0, opacity: 1 }}
            exit={prefersReducedMotion ? { opacity: 0 } : { y: 100, opacity: 0 }}
            transition={
              prefersReducedMotion
                ? { duration: 0.15 }
                : { type: 'spring', stiffness: 320, damping: 32 }
            }
            className={cn(
              'fixed inset-x-4 bottom-[calc(1.5rem+env(safe-area-inset-bottom))] z-40 mx-auto w-auto max-w-lg',
              'rounded-lg border border-fk-primary-border bg-fk-sand/95 px-4 py-3 shadow-lg backdrop-blur',
              className
            )}
          >
            <div className='flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between sm:gap-4'>
              <div className='min-w-0'>
                <p className='text-[13px] font-medium text-fk-primary-text'>{title}</p>
                {description && (
                  <p className='text-xs text-neutral-500'>{description}</p>
                )}
              </div>

              <div className='flex shrink-0 items-center gap-2'>
                {onCancel && (
                  <Button variant='ghost' size='sm' onClick={onCancel}>
                    {cancelLabel}
                  </Button>
                )}
                {actions.map((action) => (
                  <Button
                    key={action.label}
                    variant={action.variant ?? 'default'}
                    size='sm'
                    onClick={action.onClick}
                  >
                    {action.icon}
                    {action.label}
                  </Button>
                ))}
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </>
  )
}
