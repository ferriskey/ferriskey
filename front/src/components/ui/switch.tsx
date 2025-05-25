import * as SwitchPrimitive from "@radix-ui/react-switch"
import * as React from "react"

import { cn } from "@/lib/utils"
import { FormControl, FormDescription, FormItem, FormLabel } from './form'

function Switch({
  className,
  checked = false,
  ...props
}: React.ComponentProps<typeof SwitchPrimitive.Root> & { checked?: boolean }) {
  return (
    <SwitchPrimitive.Root
      data-slot="switch"
      className={cn(
        "peer data-[state=checked]:bg-primary data-[state=unchecked]:bg-input focus-visible:border-ring focus-visible:ring-ring/50 dark:data-[state=unchecked]:bg-input/80 inline-flex h-[1.15rem] w-8 shrink-0 items-center rounded-full border border-transparent shadow-xs transition-all outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50",
        className
      )}
      checked={checked}
      {...props}
    >
      <SwitchPrimitive.Thumb
        data-slot="switch-thumb"
        className={cn(
          "bg-background dark:data-[state=unchecked]:bg-foreground dark:data-[state=checked]:bg-primary-foreground pointer-events-none block size-4 rounded-full ring-0 transition-transform data-[state=checked]:translate-x-[calc(100%-2px)] data-[state=unchecked]:translate-x-0"
        )}
      />
    </SwitchPrimitive.Root>
  )
}

type FormSwitchProps = {
  label: string
  description: string | ((value: boolean) => string)
  checked?: boolean
  onChange: (checked: boolean) => void
}

function FormSwitch(props: FormSwitchProps) {
  return (
    <FormItem className="flex flex-row items-center justify-between gap-5 rounded-lg border p-3 shadow-sm">
      <div className="space-y-0.5">
        <FormLabel>{props.label}</FormLabel>
        <FormDescription>
          {typeof props.description === 'function'
            ? props.description(props.checked ?? false)
            : props.description
          }
        </FormDescription>
      </div>
      <FormControl>
        <Switch
          checked={props.checked}
          onCheckedChange={props.onChange}
        />
      </FormControl>
    </FormItem>

  )
}

export { FormSwitch, Switch }

