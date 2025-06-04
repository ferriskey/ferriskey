import { ReactNode } from "react"

export interface Value {
  label: ReactNode
  value: string
  icon?: ReactNode
  isDisabled?: boolean
  onClickEditable?: () => void
  description?: string
}
