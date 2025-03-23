export interface LoaderSpinnerProps {
  className?: string
  classWidth?: string
  classBorder?: string
  theme?: 'dark' | 'light'
}

export default function LoaderSpinner(props: LoaderSpinnerProps) {
  const {
    className = '',
    classWidth = 'w-4',
    classBorder = 'border-2',
    theme = 'light',
  } = props
  const themeClasses =
    theme === 'dark'
      ? 'border-neutral-350 border-r-neutral-50'
      : 'border-r-neutral-500'

  return (
    <div
      data-testid='spinner'
      className={`aspect-square ${classBorder || 'border-2'}  rounded-full border-solid ${themeClasses} animate-spin ${
        classWidth || 'w-4'
      } ${className || ''}`}
    />
  )
}
