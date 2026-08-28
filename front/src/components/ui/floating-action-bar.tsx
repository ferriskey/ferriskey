import { AnimatePresence, motion } from 'motion/react'
import { ReactNode, useLayoutEffect, useRef, useState } from 'react'
import { cn } from '../../lib/utils'
import { Button } from './button'

export interface FloatingAction {
  label: string
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link'
  onClick: () => void
  icon?: ReactNode
}

export interface FloatingActionBarProps {
  show: boolean
  icon?: ReactNode
  title: string
  description?: string
  onCancel?: () => void
  actions: FloatingAction[]
  cancelLabel?: string
  className?: string
}

export default function FloatingActionBar(props: FloatingActionBarProps) {
  const {
    show,
    icon,
    title,
    description,
    actions,
    onCancel,
    cancelLabel = 'Cancel',
    className = ''
  } = props
  const actionBarRef = useRef<HTMLDivElement>(null)
  const [actionBarHeight, setActionBarHeight] = useState(0)

  useLayoutEffect(() => {
    if (!show || !actionBarRef.current) return

    const actionBar = actionBarRef.current
    const updateHeight = () => setActionBarHeight(actionBar.getBoundingClientRect().height)
    const resizeObserver = new ResizeObserver(updateHeight)

    resizeObserver.observe(actionBar)

    return () => resizeObserver.disconnect()
  }, [show])

  return (
    <>
      <div
        aria-hidden='true'
        className='shrink-0 transition-[height] duration-200 ease-out motion-reduce:transition-none'
        style={{
          height: show && actionBarHeight
            ? `calc(${actionBarHeight}px + 3rem + env(safe-area-inset-bottom))`
            : 0,
        }}
      />

      <AnimatePresence>
        {show && (
          <motion.div
            ref={actionBarRef}
            initial={{ y: 100, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            exit={{ y: 100, opacity: 0 }}
            transition={{ type: 'spring', stiffness: 300, damping: 30 }}
            className={cn(
              'fixed inset-x-4 bottom-[calc(1.5rem+env(safe-area-inset-bottom))] z-50 mx-auto w-auto max-w-lg rounded-lg border bg-background px-4 py-3 shadow-lg',
              className,
            )}
          >
            <div className='flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between sm:gap-4'>
              <div className='flex min-w-0 items-center gap-3'>
                {icon && (
                  <div className='shrink-0 rounded-full bg-primary/10 p-2 text-primary'>
                    {icon}
                  </div>
                )}
                <div className='min-w-0'>
                  <p className='text-balance font-medium text-primary'>{title}</p>

                  {description && (
                    <p className='text-sm text-muted-foreground'>{description}</p>
                  )}
                </div>
              </div>

              <div className='flex w-full shrink-0 gap-2 sm:w-auto'>
                {onCancel && (
                  <Button
                    variant='ghost'
                    size='sm'
                    onClick={onCancel}
                    className='min-h-10 flex-1 transition-[color,background-color,box-shadow,transform] active:scale-[0.96] motion-reduce:transition-none motion-reduce:active:scale-100 sm:flex-none'
                  >
                    {cancelLabel}
                  </Button>
                )}

                {actions.map((action, index) => (
                  <Button
                    key={index}
                    variant={action.variant || 'default'}
                    onClick={action.onClick}
                    className='min-h-10 flex-1 transition-[color,background-color,box-shadow,transform] active:scale-[0.96] motion-reduce:transition-none motion-reduce:active:scale-100 sm:flex-none'
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
