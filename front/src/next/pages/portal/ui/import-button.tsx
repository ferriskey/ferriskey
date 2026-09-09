import { useRef } from 'react'
import { Upload } from 'lucide-react'
import { Button } from '@/components/ui/button'

export interface ImportButtonProps {
  label: string
  onImport: (file: File) => void
}

export function ImportButton({ label, onImport }: ImportButtonProps) {
  const inputRef = useRef<HTMLInputElement>(null)

  return (
    <>
      <Button variant='outline' size='sm' onClick={() => inputRef.current?.click()}>
        <Upload className='size-3.5' />
        {label}
      </Button>
      <input
        ref={inputRef}
        type='file'
        accept='application/json,.json'
        className='hidden'
        onChange={(event) => {
          const file = event.target.files?.[0]
          if (file) onImport(file)
          event.target.value = ''
        }}
      />
    </>
  )
}
