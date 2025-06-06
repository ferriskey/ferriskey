import { cn } from "@/lib/utils"
import { PropsWithChildren, ReactNode } from "react"

export interface BlockContentProps {
  title: string
  customWidth?: string
  className?: string
  classNameContent?: string
  dataTestId?: string
  headRight?: ReactNode
}

export default function BlockContent({
  children,
  className = '',
  title,
  customWidth = "w-full",
  classNameContent,
  dataTestId = "block-content",
  headRight
}: PropsWithChildren<BlockContentProps>) {

  return (
    <div
      data-testid={dataTestId}
      className={cn('mb-5 rounded border border-neutral-250 bg-neutral-100/25', customWidth, className)}
    >

      <div className="flex h-9 items-center justify-between border-b border-neutral-250 px-4">
        <h2 className="text-sm font-medium text-neutral-400">{title}</h2>
        {headRight}
      </div>

      <div className={cn('p-5', classNameContent)}>{children}</div>
    </div>
  )
}