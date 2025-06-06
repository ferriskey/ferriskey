import { UseFormReturn, useWatch } from 'react-hook-form'
import { useEffect, useRef, useState } from 'react'

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
      return data[key] !== formValues[key]
    })

    setHasChanges(isDifferent)
  }, [formValues])
  return hasChanges
}
