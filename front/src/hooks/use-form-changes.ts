import { UseFormReturn, useWatch, FieldValues } from 'react-hook-form'
import { deepEqual } from '@/utils'

export function useFormChanges<T extends FieldValues>(
  form: UseFormReturn<T>,
  originalData: T | null | undefined
): boolean {
  const formValues = useWatch({ control: form.control })

  if (!originalData || !formValues) return false

  const currentValues = formValues as Partial<T>

  return (Object.keys(originalData) as Array<keyof T>).some((key) => {
    return !deepEqual(originalData[key], currentValues[key])
  })
}
