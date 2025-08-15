import { cn } from '@/lib/utils'
import { cva, type VariantProps } from 'class-variance-authority'
import * as React from 'react'

const headingVariants = cva('transition-all', {
  variants: {
    as: {
      h1: 'h1',
      h2: 'h2',
      h3: 'h3',
      h4: 'h4',
      h5: 'h5',
      h6: 'h6',
    },
    size: {
      1: 'text-4xl',
      2: 'text-3xl',
      3: 'text-2xl',
      4: 'text-xl',
      5: 'text-lg',
      6: 'text-base',
    },
    weight: {
      light: 'font-light',
      regular: 'font-normal',
      medium: 'font-medium',
      bold: 'font-bold',
    },
    align: {
      left: 'text-left',
      center: 'text-center',
      right: 'text-right',
    },
    trim: {
      normal: 'truncate',
      start: 'truncate ...',
      end: '... truncate',
    },
    truncate: {
      true: 'truncate',
      false: 'whitespace-normal',
    },
    wrap: {
      wrap: 'break-words',
      nowrap: 'whitespace-nowrap',
      pretty: 'overflow-ellipsis',
      balance: 'line-clamp-3',
    },
    color: {
      primary: 'text-primary',
      secondary: 'text-secondary',
      danger: 'text-danger',
    },
  },
  defaultVariants: {
    as: 'h1',
    size: 1,
    weight: 'bold',
    align: 'left',
    trim: 'normal',
    truncate: false,
    wrap: 'wrap',
    color: 'primary',
  },
})


export interface HeadingProps
  extends Omit<React.HTMLAttributes<HTMLHeadingElement>, 'color'>,
    VariantProps<typeof headingVariants> {}

const Heading = React.forwardRef<HTMLHeadingElement, HeadingProps>(
  (
    {
      as = 'h1', // Default is h1
      className,
      size,
      weight,
      align,
      trim,
      truncate,
      wrap,
      color,
      ...props
    },
    ref,
  ) => {
    const computedClassName = cn(
      headingVariants({
        size,
        weight,
        align,
        trim,
        truncate,
        wrap,
        color,
        className,
      }),
    )

    // Render based on the "as" prop
    switch (as) {
      case 'h1':
        return <h1 className={computedClassName} ref={ref} {...props} />
      case 'h2':
        return <h2 className={computedClassName} ref={ref} {...props} />
      case 'h3':
        return <h3 className={computedClassName} ref={ref} {...props} />
      case 'h4':
        return <h4 className={computedClassName} ref={ref} {...props} />
      case 'h5':
        return <h5 className={computedClassName} ref={ref} {...props} />
      case 'h6':
        return <h6 className={computedClassName} ref={ref} {...props} />
      default:
        return null
    }
  },
)

Heading.displayName = 'Heading'

export { Heading }
