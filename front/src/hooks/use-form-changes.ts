import { UseFormReturn, useWatch } from 'react-hook-form'
import { useEffect, useRef, useState } from 'react'
import { deepEqual } from '@/utils'

export function useFormChanges<T extends Record<string, any>>(
  form: UseFormReturn<T>,
  originalData: T | null | undefined
): boolean {
  const [hasChanges, setHasChanges] = useState(false)
  const originalRef = useRef<T | null>(null)

  useEffect(() => {
    if (originalData) {
      originalRef.current = { ...originalData }
    }
  }, [originalData])


  const formValues = useWatch({ control: form.control })

  useEffect(() => {
    if (!originalRef.current || !formValues) return
    const data = originalRef.current
    
    const isDifferent = Object.keys(originalRef.current).some(key => {
      const areEqual = deepEqual(data[key], formValues[key])
      
      return !areEqual
    })

    setHasChanges(isDifferent)
  }, [formValues])
  return hasChanges
}
