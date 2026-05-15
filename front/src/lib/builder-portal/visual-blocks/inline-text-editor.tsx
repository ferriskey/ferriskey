import { useEffect, useRef } from 'react'

interface Props {
  content: string
  onChange: (value: string) => void
}

/**
 * Plain-text contentEditable. Portal blocks use plain strings (no rich-text
 * needed for the builder; theming handles styling).
 */
export function InlineTextEditor({ content, onChange }: Props) {
  const ref = useRef<HTMLSpanElement>(null)

  useEffect(() => {
    if (ref.current && ref.current.textContent !== content) {
      ref.current.textContent = content
    }
  }, [content])

  return (
    <span
      ref={ref}
      contentEditable
      suppressContentEditableWarning
      className='outline-none'
      onBlur={(e) => {
        const value = e.currentTarget.textContent ?? ''
        if (value !== content) onChange(value)
      }}
    />
  )
}
