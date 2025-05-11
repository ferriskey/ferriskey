import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTrigger
} from "@/components/ui/sheet"
import { Icon } from '@iconify/react'
import clsx from 'clsx'
import { Fragment, useState } from 'react'
import config from '../../../../explainer.config'

type Props = {
}

export function DocsSidebar(props: Props) {
  const [isOpen, setIsOpen] = useState(false)

  return (
    <div>
      <div>
        <SidebarRenderer />
      </div>
      <Sheet open={isOpen} onOpenChange={setIsOpen}>
        <SheetTrigger>
          <Icon
            icon={isOpen ? 'lucide:chevron-up' : 'lucide:chevron-down'}
            className="size-5"
          />
        </SheetTrigger>
        <SheetContent side="left">
          <SheetHeader>
            <SheetDescription />
            <SidebarRenderer />
          </SheetHeader>
        </SheetContent>
      </Sheet>
    </div>
  )
}

function SidebarRenderer() {
  return (
    <Fragment>
      <nav>
        <div className="space-y-3 mb-3 lg:mb-6 -mx-1 lg:mx-0">
          {
            Object.entries(config.docs).map(([key, element]) => (
              <a
                className="flex items-center gap-2 group text-primary font-medium"
                href={element.href}
              >
                <div
                  className={clsx(
                    "rounded-sm p-1 inline-flex ring-inset ring-1",
                    window.location.pathname.startsWith(element.baseUrl)
                      ? "bg-primary ring-primary text-white"
                      : "bg-gray-50 dark:bg-gray-800 ring-gray-200 dark:ring-gray-700 text-muted-foreground",
                  )}
                >
                  <Icon
                    icon={element.icon}
                    className="size-4"
                  />
                </div>
                <span
                  className={clsx(
                    "text-sm relative",
                    window.location.pathname.startsWith(element.baseUrl)
                      ? "text-primary"
                      : "text-muted-foreground",
                  )}
                >
                  {element.label}
                </span>
              </a>
            ))
          }
        </div>
      </nav>

      <hr
        className="my-5 border-dashed border-gray-200 dark:border-gray-700"
      />

      <div className="space-y-1">
        {
          currentCollectionItems.map((item) => (
            <a
              href={item.href}
              className="group block py-1.5 px-1.5 text-sm rounded-md transition-colors hover:text-primary"
            >
              <div className="flex items-center gap-2">
                {item.icon ? (
                  <Icon
                    client:only
                    icon={item.icon}
                    className={clsx(
                      "text-gray-500 dark:text-gray-400 group-hover:text-primary size-5",
                      Astro.url.pathname.startsWith(item.href)
                        ? "text-primary"
                        : "text-muted-foreground",
                    )}
                  />
                ) : (
                  <div className="size-5" />
                )}
                <span
                  className={clsx(
                    "text-sm group-hover:text-primary",
                    Astro.url.pathname.startsWith(item.href)
                      ? "text-primary"
                      : "text-muted-foreground",
                  )}
                >
                  {item.label}
                </span>
              </div>
            </a>
          ))
        }
      </div>
    </Fragment >
  )
}