import { UseFormReturn, useWatch, FieldValues } from 'react-hook-form'
import { useEffect, useRef, useState } from 'react'
import { deepEqual } from '@/utils'

export function useFormChanges<T extends FieldValues>(
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
    const currentValues = formValues as Partial<T>

    const isDifferent = (Object.keys(originalRef.current) as Array<keyof T>).some((key) => {
      const areEqual = deepEqual(data[key], currentValues[key])

      return !areEqual
    })

    setHasChanges(isDifferent)
  }, [formValues])
  return hasChanges
}
