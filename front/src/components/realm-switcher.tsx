import { useNavigate, useParams } from 'react-router'

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from '@/components/ui/sidebar'
import { Dispatch, SetStateAction, useEffect, useState } from 'react'
import { Realm } from '@/api/core.interface'
import useRealmStore from '@/store/realm.store'
import { ChevronsUpDown, Command, Map, Plus } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog.tsx'
import { InputText } from '@/components/ui/input-text.tsx'
import { Form, useForm } from 'react-hook-form'
import { z } from 'zod'
import { zodResolver } from '@hookform/resolvers/zod'
import { FormField } from '@/components/ui/form.tsx'
import { Button } from '@/components/ui/button.tsx'
import { useCreateRealm } from '@/api/realm.api.ts'
import { toast } from 'sonner'
import { REALM_OVERVIEW_URL, REALM_URL } from '@/routes/router'

export default function RealmSwitcher() {
  const { realm_name } = useParams<{ realm_name: string }>()
  const navigate = useNavigate()
  const { isMobile } = useSidebar()
  const [open, setOpen] = useState(false)
  const [dropdownOpen, setDropdownOpen] = useState(false)
  const [activeRealm, setActiveRealm] = useState<Realm | null>(null)
  const [_, setHasRealmMaster] = useState(false)
  const { userRealms } = useRealmStore()

  const handleClick = (realm: Realm) => {
    setActiveRealm(realm)
    navigate(`${REALM_URL(realm.name)}${REALM_OVERVIEW_URL}`)
  }

  useEffect(() => {
    if (userRealms && realm_name) {
      const hasMaster = userRealms.some((realm) => realm.name === 'master')
      if (hasMaster) {
        setHasRealmMaster(true)
      }
      const realm = userRealms.find((realm) => realm.name === realm_name)
      if (realm) setActiveRealm(realm)
    }
  }, [userRealms, realm_name])

  if (!activeRealm) return null

  return (
    <>
      <SidebarMenu>
        <SidebarMenuItem>
          <DropdownMenu open={dropdownOpen} onOpenChange={setDropdownOpen}>
            <DropdownMenuTrigger asChild>
              <SidebarMenuButton
                size="lg"
                className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground bg-gray-100 border shadow-2xs"
              >
                <div className="bg-white border text-sidebar-primary-foreground flex aspect-square size-10 items-center justify-center rounded-lg">
                  <Command className="text-slate-900 size-4" />
                </div>
                <div className="grid flex-1 text-left text-sm leading-tight">
                  <span className="truncate font-medium">{activeRealm?.name}</span>
                  {activeRealm.name === 'master' && (
                    <span className="text-xs text-muted-foreground">master</span>
                  )}
                </div>
                <ChevronsUpDown className="ml-auto" />
              </SidebarMenuButton>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              className="w-(--radix-dropdown-menu-trigger-width) min-w-56 rounded-lg"
              align="start"
              side={isMobile ? 'bottom' : 'right'}
              sideOffset={4}
            >
              <DropdownMenuLabel className="text-muted-foreground text-xs">
                Realms
              </DropdownMenuLabel>
              {userRealms.map((realm, index) => (
                <DropdownMenuItem
                  key={realm.name}
                  onClick={() => handleClick(realm)}
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
              <DropdownMenuItem
                className="gap-2 p-2"
                onSelect={(e) => {
                  e.preventDefault()
                  setOpen(true)
                  setDropdownOpen(false)
                }}
              >
                <div className="flex size-6 items-center justify-center rounded-md border bg-transparent">
                  <Plus className="size-4" />
                </div>
                <div className="text-muted-foreground font-medium">Create Realm</div>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      </SidebarMenu>

      <ModalCreateRealm open={open} setOpen={setOpen} />
    </>
  )
}

interface ModalCreateRealmProps {
  open: boolean
  setOpen: Dispatch<SetStateAction<boolean>>
}

const createRealmSchema = z.object({
  name: z.string().min(1, { message: 'Realm name is required' }),
})

type CreateRealmSchema = z.infer<typeof createRealmSchema>

function ModalCreateRealm({ open, setOpen }: ModalCreateRealmProps) {
  const { mutate: createRealm, data } = useCreateRealm()

  const form = useForm<CreateRealmSchema>({
    resolver: zodResolver(createRealmSchema),
    defaultValues: {
      name: '',
    },
  })

  const handleSubmit = () => {
    createRealm({
      payload: form.getValues(),
    })
  }
  const isValid = form.formState.isValid

  useEffect(() => {
    if (data) {
      toast.success(`Realm ${data.name} created successfully!`)
      setOpen(false)
    }
  }, [data])

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create Realm</DialogTitle>
          <DialogDescription>
            A realm manages a set of users, credentials, roles, and groups. A user belongs to and
            logs into a realm. Realms are isolated from one another and can only manage and
            authenticate the users that they control.
          </DialogDescription>
        </DialogHeader>
        <div>
          <Form {...form}>
            <FormField
              name="name"
              control={form.control}
              render={({ field }) => (
                <InputText
                  name={'name'}
                  label="Realm Name"
                  value={field.value}
                  onChange={field.onChange}
                />
              )}
            />

            <DialogFooter className="mt-4">
              <Button variant="destructive">Cancel</Button>
              <Button variant="outline" disabled={!isValid} onClick={handleSubmit}>
                Create Realm
              </Button>
            </DialogFooter>
          </Form>
        </div>
      </DialogContent>
    </Dialog>
  )
}
