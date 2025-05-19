import { useParams } from "react-router"

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from "@/components/ui/sidebar"
import { useEffect, useState } from "react"
import { Realm } from "@/api/api.interface"
import useRealmStore from "@/store/realm.store"
import { ChevronsUpDown, Command, Map, Plus } from "lucide-react"


export default function RealmSwitcher() {
  const { realm_name } = useParams<{ realm_name: string }>()
  const { isMobile } = useSidebar()
  const [activeRealm, setActiveRealm] = useState<Realm | null>(null)
  const { userRealms } = useRealmStore()

  useEffect(() => {
    if (userRealms && realm_name) {
      const realm = userRealms.find((realm) => realm.name === realm_name)
      if (realm) setActiveRealm(realm)
    }
  }, [userRealms, realm_name])

  if (!activeRealm) return null

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton
              size="lg"
              className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            >
              <div 
                className="bg-white border text-sidebar-primary-foreground flex aspect-square size-10 items-center justify-center rounded-lg">
                {/* <img
                  src="/logo_ferriskey.png"
                  className="size-10"
                /> */}
                <Command className="text-slate-900 size-4" />
              </div>
              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-medium">{activeRealm?.name}</span>
                {activeRealm.name === 'master' && (
                  <span className="text-xs text-muted-foreground">
                    master
                  </span>
                )}
              </div>
              <ChevronsUpDown className="ml-auto" />
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className="w-(--radix-dropdown-menu-trigger-width) min-w-56 rounded-lg"
            align="start"
            side={isMobile ? "bottom" : "right"}
            sideOffset={4}
          >
            <DropdownMenuLabel className="text-muted-foreground text-xs">
              Realms
            </DropdownMenuLabel>
            {userRealms.map((realm, index) => (
              <DropdownMenuItem
                key={realm.name}
                onClick={() => setActiveRealm(realm)}
                className="gap-2 p-2"
              >
                <div className="flex size-6 items-center justify-center rounded-md border">
                  <Map className="size-3.5 shrink-0" />
                </div>
                {realm.name}
                <DropdownMenuShortcut>⌘{index + 1}</DropdownMenuShortcut>
              </DropdownMenuItem>
            ))}
            <DropdownMenuSeparator />
            <DropdownMenuItem className="gap-2 p-2">
              <div className="flex size-6 items-center justify-center rounded-md border bg-transparent">
                <Plus className="size-4" />
              </div>
              <div className="text-muted-foreground font-medium">Add team</div>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  )
}