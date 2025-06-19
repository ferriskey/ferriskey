import { cn } from "@/lib/utils"
import { Eye, EyeClosed } from "lucide-react"
import { useEffect, useRef, useState } from "react"

export interface InputTextProps {
  name: string
  label: string
  value?: string | number
  type?: "text" | "number" | "password" | "email"
  className?: string
  onChange?: (value: string | number) => void
  error?: string
  disabled?: boolean
  // variable to control the toggle visibility of the password even if it's in disable
  togglePasswordVisibility?: boolean
}

export function InputText({
  name,
  label,
  value = "",
  onChange,
  type = "text",
  error,
  className = "",
  disabled,
  togglePasswordVisibility = false,
}: InputTextProps) {
  const [focused, setFocused] = useState<boolean>(false)
  const inputRef = useRef<HTMLDivElement>(null)
  const [currentValue, setCurrentValue] = useState<string | number>(value)
  const [currentType, setCurrentType] = useState<string>(type)

  useEffect(() => {
    setCurrentValue(value)
  }, [value, setCurrentValue])

  const hasFocus = focused
  const hasLabelUp =
    hasFocus ||
    (currentValue?.toString() && currentValue?.toString().length > 0)
      ? "input--label-up"
      : ""

  const hasError = error && error.length > 0 ? "input--error" : ""

  const inputActions = hasFocus
    ? "input--focused"
    : disabled && !togglePasswordVisibility
      ? "input--disabled"
      : ""

  return (
    <div
      className={className}
      onClick={() => inputRef.current?.querySelector("input")?.focus()}
    >
      <div className="relative">
        <div
          className={cn("input", inputActions, hasError, hasLabelUp)}
          ref={inputRef}
        >
          <div>
            <label
              htmlFor={label}
              className={cn(hasFocus ? "text-xs" : "translate-y-2 text-sm")}
            >
              {label}
            </label>

            <input
              name={name}
              id={label}
              className={`input__value`}
              type={currentType}
              disabled={disabled}
              value={currentValue}
              onChange={(e) => {
                if (onChange) onChange(e.currentTarget.value)
                setCurrentValue(e.currentTarget.value)
              }}
              onFocus={() => setFocused(true)}
              onBlur={() => setFocused(false)}
            />

            {(currentValue as string)?.length > 0 && type === "password" && (
              <div
                className="absolute right-4 top-1/2 -translate-y-1/2 text-neutral-400 transition-colors hover:text-neutral-400"
                onClick={() => {
                  console.log("Toggle password visibility");
                  
                  setCurrentType(
                    currentType === "password" ? "text" : "password"
                  )

                }}
              >
                {currentType === "password" && <Eye className="text-sm" />}
                {currentType !== "password" && (
                  <EyeClosed className="text-sm" />
                )}
              </div>
            )}
          </div>
        </div>
      </div>

      {error && (
        <p className="mt-0.5 px-3 text-xs font-medium text-red-500">{error}</p>
      )}
    </div>
  )
}
