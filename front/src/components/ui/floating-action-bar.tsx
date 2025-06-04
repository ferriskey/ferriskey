import { AnimatePresence, motion } from "motion/react"
import { ReactNode } from "react"
import { Button } from "./button"

export interface FloatingAction {
  label: string
  variant?: "default" | "destructive" | "outline" | "secondary" | "ghost" | "link"
  onClick: () => void
  icon?: ReactNode
}

export interface FloatingActionBarProps {
  show: boolean
  icon?: ReactNode
  title: string
  description?: string
  onCancel?: () => void
  actions: FloatingAction[]
  cancelLabel?: string
  className?: string

}

export default function FloatingActionBar({
  show,
  icon,
  title,
  description,
  actions,
  onCancel,
  cancelLabel = "Cancel",
  className = "",
}: FloatingActionBarProps) {

  return (
    <AnimatePresence>
      {show && (
        <motion.div
          initial={{ y: 100, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          exit={{ y: 100, opacity: 0 }}
          transition={{ type: "spring", stiffness: 300, damping: 30 }}
          className={`fixed bottom-6 left-1/2 transform -translate-x-1/2 z-50 w-full max-w-lg bg-background shadow-lg rounded-lg border px-4 py-3 ${className}`}
        >
          <div className="flex items-center justify-between gap-4">
            <div className="flex items-center gap-3">
              {icon && (
                <div className="bg-primary/10 text-primary p-2 rounded-full">
                  {icon}
                </div>
              )}
              <div>
                <p className="font-medium text-primary">{title}</p>

                {description && (
                  <p className="text-sm text-muted-foreground">{description}</p>
                )}
              </div>
            </div>

            <div className="flex gap-2">
              {onCancel && (
                <Button 
                  variant="ghost"
                  size="sm"
                  onClick={onCancel}
                >
                  {cancelLabel}
                </Button>
              )}

              {actions.map((action, index) => (
                <Button
                  key={index}
                  variant={action.variant || "default"}

                  onClick={action.onClick}
                  className="flex items-center gap-2"
                >
                  {action.icon}
                  {action.label}
                </Button>
              ))}
            </div>
          </div>

        </motion.div>
      )}
    </AnimatePresence>
  )

}