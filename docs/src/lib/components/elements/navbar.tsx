import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { NavigationMenu, NavigationMenuContent, NavigationMenuItem, NavigationMenuLink, NavigationMenuList, NavigationMenuTrigger } from "@/components/ui/navigation-menu"
import { useTheme } from "@/hooks/use-theme"
import { cn } from "@/utils"
import config from 'explainer.config'
import { LaptopIcon, MoonIcon, SunIcon } from "lucide-react"
import * as React from "react"

const ListItem = React.forwardRef<
  React.ComponentRef<"a">,
  React.ComponentPropsWithoutRef<"a">
>(({ className, title, children, ...props }, ref) => {
  return (
    <li>
      <NavigationMenuLink asChild>
        <a
          ref={ref}
          className={cn(
            "block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground",
            className
          )}
          {...props}
        >
          <div className="text-sm font-medium leading-none">{title}</div>
          <p className="line-clamp-2 text-sm leading-snug text-muted-foreground">
            {children}
          </p>
        </a>
      </NavigationMenuLink>
    </li>
  )
})
ListItem.displayName = "ListItem"

export default function Navbar() {
  const { theme, setTheme } = useTheme()
  const [open, setOpen] = React.useState(false)

  return (
    <div className="sticky top-0 z-50 w-full p-2 py-3 border-b bg-background/60 backdrop-blur-sm">
      <div className="max-w-screen-xl mx-auto flex items-center justify-between">
        {/* Logo and brand */}
        <a href="/" className="flex items-center space-x-2">
          🦀
          <span className="pl-2 font-bold text-primary text-lg">{config.meta.title}</span>
        </a>

        {/* Desktop Navigation */}
        <div className="hidden lg:block">
          <NavigationMenu>
            <NavigationMenuList>
              {config.navbar.map((element) => {
                if (element.href) {
                  return (
                    <NavigationMenuItem key={element.label}>
                      <NavigationMenuLink href={element.href} className="px-3 py-2 text-sm font-medium hover:bg-accent hover:text-accent-foreground rounded-md cursor-pointer">
                        {element.label}
                      </NavigationMenuLink>
                    </NavigationMenuItem>
                  )
                }
                return (
                  <NavigationMenuItem key={element.label}>
                    <NavigationMenuTrigger>{element.label}</NavigationMenuTrigger>
                    <NavigationMenuContent>
                      <ul className="grid w-[400px] gap-3 p-1 md:w-[500px] md:grid-cols-2 lg:w-[600px]">
                        {element.items?.map((item) => (
                          <ListItem
                            key={item.label}
                            title={item.label}
                            href={item.href}
                          >
                            {item.description}
                          </ListItem>
                        ))}
                      </ul>
                    </NavigationMenuContent>
                  </NavigationMenuItem>
                )
              })}
            </NavigationMenuList>
          </NavigationMenu>
        </div>

        {/* Search and theme toggle */}
        <div className="flex items-center space-x-2">
          <button className="p-2 rounded-md hover:bg-gray-100">
            <svg className="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
            </svg>
          </button>

          <DropdownMenu open={open} onOpenChange={setOpen}>
            <DropdownMenuTrigger asChild onClick={() => setOpen(!open)}>
              <Button variant="ghost" size="icon">
                {theme === 'light' ? <SunIcon className="h-5 w-5" /> :
                  theme === 'dark' ? <MoonIcon className="h-5 w-5" /> :
                    <LaptopIcon className="h-5 w-5" />}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-56">
              <DropdownMenuItem onClick={() => setTheme('light')}>
                <SunIcon className="mr-2 h-4 w-4" />
                Light
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => setTheme('dark')}>
                <MoonIcon className="mr-2 h-4 w-4" />
                Dark
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => setTheme('system')}>
                <LaptopIcon className="mr-2 h-4 w-4" />
                System
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          {config.urls.github && (
            <a
              href={config.urls.github}
              rel="noopener noreferrer"
              target="_blank"
              className="rounded-[calc(var(--ui-radius)*1.5)] font-medium inline-flex items-center disabled:cursor-not-allowed aria-disabled:cursor-not-allowed disabled:opacity-75 aria-disabled:opacity-75 transition-colors px-2.5 py-1.5 text-sm gap-1.5 text-(--ui-text) hover:bg-(--ui-bg-elevated) focus:outline-none focus-visible:bg-(--ui-bg-elevated) hover:disabled:bg-transparent dark:hover:disabled:bg-transparent hover:aria-disabled:bg-transparent dark:hover:aria-disabled:bg-transparent"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width={24}
                height={24}
                viewBox="0 0 24 24"
              >
                <path fill="currentColor" d="M12 .297c-6.63 0-12 5.373-12 12c0 5.303 3.438 9.8 8.205 11.385c.6.113.82-.258.82-.577c0-.285-.01-1.04-.015-2.04c-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729c1.205.084 1.838 1.236 1.838 1.236c1.07 1.835 2.809 1.305 3.495.998c.108-.776.417-1.305.76-1.605c-2.665-.3-5.466-1.332-5.466-5.93c0-1.31.465-2.38 1.235-3.22c-.135-.303-.54-1.523.105-3.176c0 0 1.005-.322 3.3 1.23c.96-.267 1.98-.399 3-.405c1.02.006 2.04.138 3 .405c2.28-1.552 3.285-1.23 3.285-1.23c.645 1.653.24 2.873.12 3.176c.765.84 1.23 1.91 1.23 3.22c0 4.61-2.805 5.625-5.475 5.92c.42.36.81 1.096.81 2.22c0 1.606-.015 2.896-.015 3.286c0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"></path>
              </svg>
              <span className="sr-only">{config.meta.title} on GitHub</span>
            </a>
          )}
        </div>
      </div>
    </div>
  )
}
