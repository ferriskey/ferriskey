import { AnimatePresence, motion, useReducedMotion } from 'motion/react'
import { ReactNode, useLayoutEffect, useRef, useState } from 'react'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
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
            ? `calc(${barHeight}px + 1rem + env(safe-area-inset-bottom))`
            : 0,
        }}
      />

      <AnimatePresence>
        {show && (
          <motion.div
            ref={barRef}
            initial={prefersReducedMotion ? { opacity: 0 } : { y: 64, opacity: 0 }}
            animate={prefersReducedMotion ? { opacity: 1 } : { y: 0, opacity: 1 }}
            exit={prefersReducedMotion ? { opacity: 0 } : { y: 64, opacity: 0 }}
            transition={
              prefersReducedMotion
                ? { duration: 0.15 }
                : { type: 'spring', stiffness: 320, damping: 32 }
            }
            className={cn(
              'fixed inset-x-0 bottom-0 z-40 border-t border-fk-primary-border bg-fk-sand/95 backdrop-blur',
              'pb-[env(safe-area-inset-bottom)]',
              className
            )}
          >
            <div
              className={cn(
                'mx-auto flex flex-wrap items-center justify-between gap-x-4 gap-y-2 px-5 py-2.5',
                tokens.page.maxWidth
              )}
            >
              <p className='min-w-0 text-[13px]'>
                <span className='font-medium text-fk-primary-text'>{title}</span>
                {description && (
                  <span className='ml-2 text-neutral-500'>{description}</span>
                )}
              </p>

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
