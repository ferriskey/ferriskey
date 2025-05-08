import * as React from "react"
import { cn } from "@/utils"
import { Menu, X } from "lucide-react"
import { Button } from "../ui/button"

interface SidebarItem {
  title: string
  href: string
  icon?: React.ReactNode
  items?: SidebarItem[]
}

interface DocsSidebarProps {
  items: SidebarItem[]
  className?: string
}

const SidebarLink = ({ item }: { item: SidebarItem }) => {
  return (
    <li>
      <a
        href={item.href}
        className="flex items-center gap-2 px-3 py-2 text-sm text-muted-foreground rounded-md hover:bg-accent hover:text-accent-foreground"
      >
        {item.icon}
        {item.title}
      </a>
      {item.items && (
        <ul className="ml-4 mt-2 space-y-1">
          {item.items.map((subItem) => (
            <SidebarLink key={subItem.href} item={subItem} />
          ))}
        </ul>
      )}
    </li>
  )
}

export function DocsSidebar({ items, className }: DocsSidebarProps) {
  const [isOpen, setIsOpen] = React.useState(false)

  return (
    <>
      {/* Mobile toggle */}
      <Button
        variant="ghost"
        size="icon"
        className="fixed top-4 left-4 z-40 lg:hidden"
        onClick={() => setIsOpen(!isOpen)}
      >
        {isOpen ? <X className="h-6 w-6" /> : <Menu className="h-6 w-6" />}
      </Button>

      {/* Backdrop */}
      {isOpen && (
        <div
          className="fixed inset-0 z-30 bg-background/80 backdrop-blur-sm lg:hidden"
          onClick={() => setIsOpen(false)}
        />
      )}

      {/* Sidebar */}
      <aside
        className={cn(
          "fixed top-0 left-0 z-30 h-screen w-72 border-r bg-background transition-transform lg:translate-x-0 lg:relative",
          isOpen ? "translate-x-0" : "-translate-x-full",
          className
        )}
      >
        <div className="sticky top-0 overflow-y-auto h-full p-6 pt-16 lg:pt-6">
          <nav>
            <ul className="space-y-2">
              {items.map((item) => (
                <SidebarLink key={item.href} item={item} />
              ))}
            </ul>
          </nav>
        </div>
      </aside>
    </>
  )
} 