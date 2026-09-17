import { useEffect, useMemo, useState } from 'react'
import { useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { ChevronDown, ChevronRight, Plus, Search, Trash2 } from 'lucide-react'

import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { EntityAvatar } from '@/components/ui/entity-avatar'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { DataTable, ColumnDef } from '@/components/ui/data-table'
import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import MultipleSelector, { Option } from '@/components/ui/multiselect'
import { useGetUsers } from '@/api/user.api'
import { useGetRoles } from '@/api/role.api'
import { apiErrorMessage } from '@/lib/api-error'
import {
  GroupNode,
  useAddGroupMember,
  useAssignGroupRole,
  useCreateGroup,
  useDeleteGroup,
  useDeleteGroupAttribute,
  useGroupAttributes,
  useGroupMembers,
  useGroupRoles,
  useGroups,
  useRemoveGroupMember,
  useRevokeGroupRole,
  useUpsertGroupAttribute,
} from '@/api/group.api'

const PAGE_SIZE = 50

const EMPTY_VALUE = '—'

const MEMBER_SEARCH_KEYS: (keyof Schemas.User)[] = ['username', 'email']

const GROUP_TAB = {
  members: 'members',
  roles: 'roles',
  attributes: 'attributes',
  subgroups: 'subgroups',
} as const

const fail = (e: unknown) => toast.error(apiErrorMessage(e, 'Request failed'))

/** Find a node by id in the group tree (so the detail stays fresh after refetches). */
function findNode(nodes: GroupNode[], id: string): GroupNode | undefined {
  for (const node of nodes) {
    if (node.id === id) return node
    const found = findNode(node.children, id)
    if (found) return found
  }
  return undefined
}

/* ------------------------------------------------------------------ tree --- */

interface TreeProps {
  nodes: GroupNode[]
  depth: number
  selectedId?: string
  onSelect: (node: GroupNode) => void
  onAddChild: (parentId: string) => void
  onDelete: (node: GroupNode) => void
}

function GroupTree({ nodes, depth, selectedId, onSelect, onAddChild, onDelete }: TreeProps) {
  const { t } = useTranslation('organization')
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({})

  return (
    <div className='flex flex-col'>
      {nodes.map((node) => {
        const hasChildren = node.children.length > 0
        const isCollapsed = collapsed[node.id]
        return (
          <div key={node.id}>
            <div
              className={`group flex items-center gap-1 rounded-md px-2 py-1.5 text-sm ${
                selectedId === node.id ? 'bg-primary/10 text-primary' : 'hover:bg-muted'
              }`}
              style={{ paddingLeft: `${depth * 16 + 8}px` }}
            >
              <button
                type='button'
                className='shrink-0 text-muted-foreground'
                onClick={() => setCollapsed((c) => ({ ...c, [node.id]: !c[node.id] }))}
                aria-label={t('groups.tree.toggle')}
              >
                {hasChildren ? (
                  isCollapsed ? (
                    <ChevronRight className='h-4 w-4' />
                  ) : (
                    <ChevronDown className='h-4 w-4' />
                  )
                ) : (
                  <span className='inline-block w-4' />
                )}
              </button>
              <button
                type='button'
                className='flex-1 truncate text-left'
                onClick={() => onSelect(node)}
              >
                {node.name}
              </button>
              <button
                type='button'
                className='shrink-0 text-muted-foreground opacity-0 transition-opacity hover:text-foreground group-hover:opacity-100'
                title={t('groups.tree.add_child')}
                onClick={() => onAddChild(node.id)}
              >
                <Plus className='h-3.5 w-3.5' />
              </button>
              <button
                type='button'
                className='shrink-0 text-muted-foreground opacity-0 transition-opacity hover:text-destructive group-hover:opacity-100'
                title={t('groups.tree.delete')}
                onClick={() => onDelete(node)}
              >
                <Trash2 className='h-3.5 w-3.5' />
              </button>
            </div>
            {hasChildren && !isCollapsed && (
              <GroupTree
                nodes={node.children}
                depth={depth + 1}
                selectedId={selectedId}
                onSelect={onSelect}
                onAddChild={onAddChild}
                onDelete={onDelete}
              />
            )}
          </div>
        )
      })}
    </div>
  )
}

/* --------------------------------------------------------------- members --- */

function AddMembersDialog({
  realm,
  orgId,
  groupId,
}: {
  realm?: string
  orgId?: string
  groupId: string
}) {
  const { t } = useTranslation('organization')
  const [open, setOpen] = useState(false)
  const [selected, setSelected] = useState<Schemas.User[]>([])
  const { data: usersResp } = useGetUsers({ realm })
  const users = usersResp?.data ?? []
  const addMember = useAddGroupMember(realm, orgId, groupId)

  const columns: ColumnDef<Schemas.User>[] = [
    {
      id: 'username',
      header: t('groups.members.add.columns.username'),
      cell: (u) => (
        <div className='flex flex-col'>
          <span className='font-medium'>{u.username}</span>
          <span className='text-xs text-muted-foreground'>{u.email ?? EMPTY_VALUE}</span>
        </div>
      ),
    },
    {
      id: 'name',
      header: t('groups.members.add.columns.name'),
      cell: (u) => (
        <span className='text-sm text-muted-foreground'>
          {[u.firstname, u.lastname].filter(Boolean).join(' ') || EMPTY_VALUE}
        </span>
      ),
    },
  ]

  const submit = async () => {
    if (selected.length === 0) return
    // Add each selected user; a 409 means "already a member" — treat as a no-op.
    const results = await Promise.allSettled(
      selected.map((u) => addMember.mutateAsync(u.id))
    )
    const failed = results.filter(
      (r) => r.status === 'rejected' && !/already/i.test(String((r as PromiseRejectedResult).reason))
    )
    if (failed.length > 0) {
      toast.error(t('groups.members.add.failed', { count: failed.length }))
    } else {
      toast.success(t('groups.members.add.added', { count: selected.length }))
    }
    setSelected([])
    setOpen(false)
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size='sm'>
          <Plus className='mr-1 h-4 w-4' /> {t('groups.members.add.trigger')}
        </Button>
      </DialogTrigger>
      <DialogContent className='max-w-4xl'>
        <DialogTitle>{t('groups.members.add.title')}</DialogTitle>
        <DialogBody>
          <DataTable
            columns={columns}
            data={users}
            enableSelection
            searchKeys={MEMBER_SEARCH_KEYS}
            searchPlaceholder={t('groups.members.add.search_placeholder')}
            onSelectionChange={setSelected}
          />
        </DialogBody>
        <DialogFooter>
          <Button variant='ghost' onClick={() => setOpen(false)}>
            {t('groups.members.add.cancel')}
          </Button>
          <Button disabled={selected.length === 0} onClick={submit}>
            {selected.length > 0
              ? t('groups.members.add.submit_count', { total: selected.length })
              : t('groups.members.add.submit')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function MembersTab({ realm, orgId, group }: { realm?: string; orgId?: string; group: GroupNode }) {
  const { t } = useTranslation('organization')
  const [search, setSearch] = useState('')
  const [debounced, setDebounced] = useState('')
  const [offset, setOffset] = useState(0)

  // Debounce the search box and reset to the first page on a new term.
  useEffect(() => {
    const t = setTimeout(() => {
      setDebounced(search)
      setOffset(0)
    }, 300)
    return () => clearTimeout(t)
  }, [search])

  const { data, isLoading } = useGroupMembers(realm, orgId, group.id, {
    limit: PAGE_SIZE,
    offset,
    search: debounced,
  })
  const removeMember = useRemoveGroupMember(realm, orgId, group.id)

  const members = data?.data ?? []
  const total = data?.total ?? 0
  const from = total === 0 ? 0 : offset + 1
  const to = offset + members.length

  return (
    <div className='flex flex-col gap-3'>
      {/* Header — mirrors the OverviewList listings (roles/clients/users) */}
      <div className='flex items-center justify-between'>
        <h2 className='text-base font-semibold'>
          {t('groups.members.title', { total })}
        </h2>
        <div className='flex items-center gap-2'>
          <div className='relative'>
            <Search className='absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground' />
            <Input
              type='search'
              placeholder={t('groups.members.search_placeholder')}
              className='h-9 w-64 bg-background pl-9 text-sm'
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
          <AddMembersDialog realm={realm} orgId={orgId} groupId={group.id} />
        </div>
      </div>

      {/* List body */}
      <div className='overflow-hidden rounded-md border'>
        {isLoading ? (
          <div className='flex h-24 items-center justify-center text-sm text-muted-foreground'>
            {t('groups.members.loading')}
          </div>
        ) : members.length === 0 ? (
          <div className='flex h-24 items-center justify-center text-sm text-muted-foreground'>
            {debounced ? t('groups.members.no_match') : t('groups.members.empty')}
          </div>
        ) : (
          members.map((m) => (
            <div
              key={m.id}
              className='flex items-center justify-between border-b px-4 py-3 transition-colors last:border-b-0 hover:bg-muted/40'
            >
              <div className='flex items-center gap-3'>
                <EntityAvatar size='sm' label={m.firstname || m.username} />
                <div>
                  <div className='text-sm font-medium'>
                    {[m.firstname, m.lastname].filter(Boolean).join(' ') || m.username}
                  </div>
                  <div className='text-xs text-muted-foreground'>{m.email ?? m.username}</div>
                </div>
              </div>
              <div className='flex items-center gap-3'>
                <Badge variant={m.enabled ? 'default' : 'secondary'}>
                  {m.enabled
                    ? t('groups.members.status.enabled')
                    : t('groups.members.status.disabled')}
                </Badge>
                <button
                  type='button'
                  className='text-muted-foreground hover:text-destructive'
                  title={t('groups.members.remove')}
                  onClick={() => removeMember.mutate(m.user_id, { onError: fail })}
                >
                  <Trash2 className='h-4 w-4' />
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Pagination */}
      {total > PAGE_SIZE && (
        <div className='flex items-center justify-between px-1'>
          <span className='text-sm text-muted-foreground'>
            {t('groups.members.pagination.range', { from, to, total })}
          </span>
          <div className='flex items-center gap-1'>
            <Button
              variant='outline'
              size='sm'
              className='h-8'
              disabled={offset === 0}
              onClick={() => setOffset(Math.max(0, offset - PAGE_SIZE))}
            >
              {t('groups.members.pagination.previous')}
            </Button>
            <Button
              variant='outline'
              size='sm'
              className='h-8'
              disabled={to >= total}
              onClick={() => setOffset(offset + PAGE_SIZE)}
            >
              {t('groups.members.pagination.next')}
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}

/* ----------------------------------------------------------------- roles --- */

function RolesTab({ realm, orgId, group }: { realm?: string; orgId?: string; group: GroupNode }) {
  const { t } = useTranslation('organization')
  const { data: assigned } = useGroupRoles(realm, orgId, group.id)
  const rolesResp = useGetRoles({ realm })
  const rolesData = (rolesResp.data as { data?: Array<{ id: string; name: string }> } | undefined)
    ?.data
  const assignRole = useAssignGroupRole(realm, orgId, group.id)
  const revokeRole = useRevokeGroupRole(realm, orgId, group.id)

  const options: Option[] = useMemo(
    () => (rolesData ?? []).map((r) => ({ value: r.id, label: r.name })),
    [rolesData]
  )
  const value: Option[] = useMemo(
    () => (assigned ?? []).map((r) => ({ value: r.id, label: r.name })),
    [assigned]
  )

  const onChange = (next: Option[]) => {
    const before = new Set(value.map((o) => o.value))
    const after = new Set(next.map((o) => o.value))
    next.filter((o) => !before.has(o.value)).forEach((o) =>
      assignRole.mutate(o.value, { onError: fail })
    )
    value.filter((o) => !after.has(o.value)).forEach((o) =>
      revokeRole.mutate(o.value, { onError: fail })
    )
  }

  return (
    <div className='flex flex-col gap-3'>
      <p className='text-sm text-muted-foreground'>{t('groups.roles.description')}</p>
      <MultipleSelector
        value={value}
        options={options}
        onChange={onChange}
        placeholder={t('groups.roles.search_placeholder')}
        hidePlaceholderWhenSelected
        emptyIndicator={
          <p className='text-center text-sm text-muted-foreground'>{t('groups.roles.empty')}</p>
        }
      />
    </div>
  )
}

/* ------------------------------------------------------------ attributes --- */

function AttributesTab({
  realm,
  orgId,
  group,
}: {
  realm?: string
  orgId?: string
  group: GroupNode
}) {
  const { t } = useTranslation('organization')
  const { data: attributes } = useGroupAttributes(realm, orgId, group.id)
  const upsert = useUpsertGroupAttribute(realm, orgId, group.id)
  const remove = useDeleteGroupAttribute(realm, orgId, group.id)
  const [key, setKey] = useState('')
  const [value, setValue] = useState('')

  const save = () => {
    if (!key || !value) return
    upsert.mutate(
      { key, value },
      {
        onError: fail,
        onSuccess: () => {
          setKey('')
          setValue('')
        },
      }
    )
  }

  return (
    <div className='flex flex-col gap-3'>
      <div className='rounded-md border'>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t('groups.attributes.columns.key')}</TableHead>
              <TableHead>{t('groups.attributes.columns.value')}</TableHead>
              <TableHead className='w-10' />
            </TableRow>
          </TableHeader>
          <TableBody>
            {(attributes ?? []).length === 0 ? (
              <TableRow>
                <TableCell colSpan={3} className='text-center text-sm text-muted-foreground'>
                  {t('groups.attributes.empty')}
                </TableCell>
              </TableRow>
            ) : (
              (attributes ?? []).map((a) => (
                <TableRow key={a.id}>
                  <TableCell className='font-mono text-sm'>{a.key}</TableCell>
                  <TableCell className='text-sm'>{a.value}</TableCell>
                  <TableCell>
                    <button
                      type='button'
                      className='text-muted-foreground hover:text-destructive'
                      onClick={() => remove.mutate(a.key, { onError: fail })}
                    >
                      <Trash2 className='h-4 w-4' />
                    </button>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>
      <div className='flex items-center gap-2'>
        <Input
          placeholder={t('groups.attributes.key_placeholder')}
          value={key}
          onChange={(e) => setKey(e.target.value)}
        />
        <Input
          placeholder={t('groups.attributes.value_placeholder')}
          value={value}
          onChange={(e) => setValue(e.target.value)}
        />
        <Button variant='outline' disabled={!key || !value} onClick={save}>
          {t('groups.attributes.add')}
        </Button>
      </div>
    </div>
  )
}

/* ------------------------------------------------------------- subgroups --- */

function SubGroupsTab({
  realm,
  orgId,
  group,
  onSelect,
}: {
  realm?: string
  orgId?: string
  group: GroupNode
  onSelect: (id: string) => void
}) {
  const { t } = useTranslation('organization')
  const createGroup = useCreateGroup(realm, orgId)
  const [name, setName] = useState('')

  const create = () => {
    if (!name) return
    createGroup.mutate(
      { name, parent_group_id: group.id },
      { onError: fail, onSuccess: () => setName('') }
    )
  }

  return (
    <div className='flex flex-col gap-3'>
      <div className='flex items-center gap-2'>
        <Input
          placeholder={t('groups.subgroups.name_placeholder')}
          value={name}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && create()}
        />
        <Button variant='outline' disabled={!name} onClick={create}>
          {t('groups.subgroups.add')}
        </Button>
      </div>
      <div className='flex flex-col divide-y rounded-md border'>
        {group.children.length === 0 ? (
          <p className='px-3 py-4 text-center text-sm text-muted-foreground'>
            {t('groups.subgroups.empty')}
          </p>
        ) : (
          group.children.map((child) => (
            <button
              key={child.id}
              type='button'
              className='px-3 py-2 text-left text-sm hover:bg-muted'
              onClick={() => onSelect(child.id)}
            >
              {child.name}
              {child.children.length > 0 && (
                <span className='ml-2 text-xs text-muted-foreground'>
                  {t('groups.subgroups.child_count', { total: child.children.length })}
                </span>
              )}
            </button>
          ))
        )}
      </div>
    </div>
  )
}

/* ---------------------------------------------------------------- detail --- */

function GroupDetail({
  realm,
  orgId,
  group,
  onSelect,
}: {
  realm?: string
  orgId?: string
  group: GroupNode
  onSelect: (id: string) => void
}) {
  const { t } = useTranslation('organization')

  return (
    <div className='flex flex-col gap-4'>
      <div>
        <h3 className='text-base font-semibold'>{group.name}</h3>
        {group.description && (
          <p className='text-sm text-muted-foreground'>{group.description}</p>
        )}
      </div>

      <Tabs defaultValue={GROUP_TAB.members}>
        <TabsList>
          <TabsTrigger value={GROUP_TAB.members}>{t('groups.tabs.members')}</TabsTrigger>
          <TabsTrigger value={GROUP_TAB.roles}>{t('groups.tabs.roles')}</TabsTrigger>
          <TabsTrigger value={GROUP_TAB.attributes}>{t('groups.tabs.attributes')}</TabsTrigger>
          <TabsTrigger value={GROUP_TAB.subgroups}>{t('groups.tabs.subgroups')}</TabsTrigger>
        </TabsList>
        <TabsContent value={GROUP_TAB.members} className='pt-4'>
          <MembersTab realm={realm} orgId={orgId} group={group} />
        </TabsContent>
        <TabsContent value={GROUP_TAB.roles} className='pt-4'>
          <RolesTab realm={realm} orgId={orgId} group={group} />
        </TabsContent>
        <TabsContent value={GROUP_TAB.attributes} className='pt-4'>
          <AttributesTab realm={realm} orgId={orgId} group={group} />
        </TabsContent>
        <TabsContent value={GROUP_TAB.subgroups} className='pt-4'>
          <SubGroupsTab realm={realm} orgId={orgId} group={group} onSelect={onSelect} />
        </TabsContent>
      </Tabs>
    </div>
  )
}

/* ------------------------------------------------------------------ page --- */

export default function PageOrganizationGroupsFeature() {
  const { realm_name, organizationId } = useParams<
    RouterParams & { organizationId: string }
  >()
  const { t } = useTranslation('organization')
  const { data: tree, isLoading } = useGroups(realm_name, organizationId)
  const createGroup = useCreateGroup(realm_name, organizationId)
  const deleteGroup = useDeleteGroup(realm_name, organizationId)

  const [selectedId, setSelectedId] = useState<string | undefined>()
  const [newName, setNewName] = useState('')

  // Sub-group creation dialog: holds the target parent id (open when set) and its draft name.
  const [addParentId, setAddParentId] = useState<string | undefined>()
  const [childName, setChildName] = useState('')

  // Group being deleted (open the confirmation dialog when set).
  const [deleteTarget, setDeleteTarget] = useState<GroupNode | undefined>()

  // Re-derive the selected node from the freshest tree so it updates after mutations.
  const selected = useMemo(
    () => (selectedId ? findNode(tree ?? [], selectedId) : undefined),
    [tree, selectedId]
  )

  const addParent = useMemo(
    () => (addParentId ? findNode(tree ?? [], addParentId) : undefined),
    [tree, addParentId]
  )

  const createRoot = () => {
    if (!newName) return
    createGroup.mutate({ name: newName }, { onError: fail, onSuccess: () => setNewName('') })
  }

  const addChild = (parentId: string) => {
    setChildName('')
    setAddParentId(parentId)
  }

  const submitChild = () => {
    if (!childName || !addParentId) return
    createGroup.mutate(
      { name: childName, parent_group_id: addParentId },
      {
        onError: fail,
        onSuccess: () => {
          setChildName('')
          setAddParentId(undefined)
        },
      }
    )
  }

  const remove = (node: GroupNode) => {
    setDeleteTarget(node)
  }

  const confirmDelete = () => {
    if (!deleteTarget) return
    const node = deleteTarget
    deleteGroup.mutate(node.id, {
      onError: fail,
      onSuccess: () => {
        setSelectedId((id) => (id === node.id ? undefined : id))
        setDeleteTarget(undefined)
      },
    })
  }

  return (
    <div className='grid grid-cols-1 gap-6 md:grid-cols-[320px_1fr]'>
      <div className='flex flex-col gap-3 rounded-md border p-3'>
        <div className='flex items-center gap-2'>
          <Input
            placeholder={t('groups.tree.new_placeholder')}
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && createRoot()}
          />
          <Button variant='outline' disabled={!newName} onClick={createRoot}>
            {t('groups.tree.add')}
          </Button>
        </div>
        {isLoading ? (
          <p className='text-sm text-muted-foreground'>{t('groups.tree.loading')}</p>
        ) : (tree ?? []).length === 0 ? (
          <p className='text-sm text-muted-foreground'>{t('groups.tree.empty')}</p>
        ) : (
          <GroupTree
            nodes={tree ?? []}
            depth={0}
            selectedId={selectedId}
            onSelect={(n) => setSelectedId(n.id)}
            onAddChild={addChild}
            onDelete={remove}
          />
        )}
      </div>

      <div className='rounded-md border p-4'>
        {selected ? (
          <GroupDetail
            realm={realm_name}
            orgId={organizationId}
            group={selected}
            onSelect={setSelectedId}
          />
        ) : (
          <p className='text-sm text-muted-foreground'>{t('groups.no_selection')}</p>
        )}
      </div>

      <Dialog
        open={addParentId !== undefined}
        onOpenChange={(open) => !open && setAddParentId(undefined)}
      >
        <DialogContent className='!max-w-md'>
          <DialogHeader>
            <DialogTitle>{t('groups.create.title')}</DialogTitle>
            <DialogDescription>
              {addParent
                ? t('groups.create.description', { name: addParent.name })
                : t('groups.create.description_unknown')}
            </DialogDescription>
          </DialogHeader>
          <DialogBody>
            <Input
              autoFocus
              placeholder={t('groups.create.name_placeholder')}
              value={childName}
              onChange={(e) => setChildName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && submitChild()}
            />
          </DialogBody>
          <DialogFooter>
            <Button variant='ghost' onClick={() => setAddParentId(undefined)}>
              {t('groups.create.cancel')}
            </Button>
            <Button disabled={!childName || createGroup.isPending} onClick={submitChild}>
              {t('groups.create.submit')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        open={deleteTarget !== undefined}
        onOpenChange={(open) => !open && setDeleteTarget(undefined)}
      >
        <DialogContent className='!max-w-md'>
          <DialogHeader>
            <DialogTitle>{t('groups.delete.title')}</DialogTitle>
            <DialogDescription>
              {deleteTarget ? t('groups.delete.description', { name: deleteTarget.name }) : ''}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant='ghost' onClick={() => setDeleteTarget(undefined)}>
              {t('groups.delete.cancel')}
            </Button>
            <Button
              variant='destructive'
              disabled={deleteGroup.isPending}
              onClick={confirmDelete}
            >
              {t('groups.delete.submit')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
