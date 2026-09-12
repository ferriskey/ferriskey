import type { ComponentProps, ReactNode } from 'react'
import { X } from 'lucide-react'
import { Dialog as DialogPrimitive } from 'radix-ui'
import { cn } from '@/lib/utils'

function Sheet({ ...props }: ComponentProps<typeof DialogPrimitive.Root>) {
  return <DialogPrimitive.Root data-slot='sheet' {...props} />
}

function SheetTitle({ className, ...props }: ComponentProps<typeof DialogPrimitive.Title>) {
  return (
    <DialogPrimitive.Title
      data-slot='sheet-title'
      className={cn('text-[13px] font-semibold tracking-tight', className)}
      {...props}
    />
  )
}

function SheetContent({
  className,
  children,
  label,
  ...props
}: ComponentProps<typeof DialogPrimitive.Content> & { label: ReactNode }) {
  return (
    <DialogPrimitive.Portal>
      <DialogPrimitive.Overlay
        data-slot='sheet-overlay'
        className='fixed inset-0 z-50 bg-black/30 [backdrop-filter:blur(4px)] data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0'
      />
      <DialogPrimitive.Content
        data-slot='sheet-content'
        className={cn(
          'fixed inset-y-0 left-0 z-50 flex w-[17rem] max-w-[85vw] flex-col border-r border-fk-line bg-white outline-0 dark:bg-fk-surface',
          'pl-[env(safe-area-inset-left)] pb-[env(safe-area-inset-bottom)]',
          'duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:slide-out-to-left data-[state=open]:slide-in-from-left',
          className,
        )}
        {...props}
      >
        <div className='flex h-11 shrink-0 items-center justify-between border-b border-fk-line px-3'>
          <SheetTitle>{label}</SheetTitle>
          <DialogPrimitive.Close
            aria-label='Close navigation'
            className='grid size-7 cursor-pointer place-items-center rounded-md text-neutral-400 transition-colors hover:bg-neutral-100 hover:text-neutral-900 dark:text-neutral-500 dark:hover:bg-fk-raised dark:hover:text-neutral-100'
          >
            <X className='size-4' />
          </DialogPrimitive.Close>
        </div>
        <div className='min-h-0 flex-1 overflow-y-auto'>{children}</div>
      </DialogPrimitive.Content>
    </DialogPrimitive.Portal>
  )
}

export { Sheet, SheetContent }
