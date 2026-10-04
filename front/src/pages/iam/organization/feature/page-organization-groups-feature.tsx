import { useEffect, useMemo, useState } from 'react'
import { useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { FolderTree, Plus, Search, Trash2 } from 'lucide-react'

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
import {
  DataView,
  FilterBar,
  PaginationBar,
  usePagedListing,
  type Column,
  type FilterField,
  type PagedListing,
  type ViewMode,
} from '@/components/kit'
import { useUserSearch } from '@/api/user.api'
import { useRoleSearch } from '@/api/role.api'
import { apiErrorMessage } from '@/lib/api-error'
import { formatRelative } from '@/utils/format-date'
import { groupRelationSource } from '@/api/group.relation'
import {
  GROUP_FILTER_KEYS,
  GROUP_SEARCH_LIMIT,
  useAddGroupMember,
  useAssignGroupRole,
  useCreateGroup,
  useDeleteGroup,
  useDeleteGroupAttribute,
  useGroup,
  useGroupAttributes,
  useGroupMembers,
  useGroupRoles,
  useGroups,
  useGroupsByIds,
  useRemoveGroupMember,
  useRevokeGroupRole,
  useUpsertGroupAttribute,
  type Group,
  type GroupsQuery,
} from '@/api/group.api'

const PAGE_SIZE = 50

const EMPTY_VALUE = '—'

const GROUPS_VIEW: ViewMode = 'list'

const GROUP_TAB = {
  members: 'members',
  roles: 'roles',
  attributes: 'attributes',
  subgroups: 'subgroups',
} as const

const fail = (e: unknown) => toast.error(apiErrorMessage(e, 'Request failed'))

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
  const { search, setSearch, users } = useUserSearch({ realm })
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
          <div className='relative mb-3'>
            <Search className='absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground' />
            <Input
              type='search'
              placeholder={t('groups.members.add.search_placeholder')}
              className='h-9 w-64 bg-background pl-9 text-sm'
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
          <DataTable
            columns={columns}
            data={users}
            enableSelection
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

function MembersTab({ realm, orgId, group }: { realm?: string; orgId?: string; group: Group }) {
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

function RolesTab({ realm, orgId, group }: { realm?: string; orgId?: string; group: Group }) {
  const { t } = useTranslation('organization')
  const { data: assigned } = useGroupRoles(realm, orgId, group.id)
  const { roles: found, setSearch } = useRoleSearch({ realm })
  const assignRole = useAssignGroupRole(realm, orgId, group.id)
  const revokeRole = useRevokeGroupRole(realm, orgId, group.id)

  const options: Option[] = useMemo(
    () => found.map((r) => ({ value: r.id, label: r.name })),
    [found]
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
        inputProps={{ onValueChange: setSearch }}
        placeholder={t('groups.roles.search_placeholder')}
        hidePlaceholderWhenSelected
        emptyIndicator={
          <p className='text-center text-sm text-muted-foreground'>{t('groups.roles.empty')}</p>
        }
      />
    </div>
  )
}

function AttributesTab({
  realm,
  orgId,
  group,
}: {
  realm?: string
  orgId?: string
  group: Group
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

function SubGroupsTab({
  realm,
  orgId,
  group,
  onSelect,
  onBrowse,
}: {
  realm?: string
  orgId?: string
  group: Group
  onSelect: (id: string) => void
  onBrowse: (id: string) => void
}) {
  const { t } = useTranslation('organization')
  const createGroup = useCreateGroup(realm, orgId)
  const [name, setName] = useState('')
  const [page, setPage] = useState(1)
  const { data } = useGroups({
    realm,
    orgId,
    query: {
      parent_group_id: group.id,
      order_by: 'name',
      order: 'asc',
      limit: GROUP_SEARCH_LIMIT,
      page,
    },
  })
  const children = data?.data ?? []
  const pagination = data?.metadata

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
        <Button variant='ghost' onClick={() => onBrowse(group.id)}>
          {t('groups.subgroups.browse')}
        </Button>
      </div>
      <div className='flex flex-col divide-y rounded-md border'>
        {children.length === 0 ? (
          <p className='px-3 py-4 text-center text-sm text-muted-foreground'>
            {t('groups.subgroups.empty')}
          </p>
        ) : (
          children.map((child) => (
            <button
              key={child.id}
              type='button'
              className='px-3 py-2 text-left text-sm hover:bg-muted'
              onClick={() => onSelect(child.id)}
            >
              {child.name}
            </button>
          ))
        )}
      </div>
      {pagination && pagination.total > pagination.limit && (
        <PaginationBar pagination={pagination} onPageChange={setPage} />
      )}
    </div>
  )
}

function GroupDetail({
  realm,
  orgId,
  group,
  onSelect,
  onBrowse,
}: {
  realm?: string
  orgId?: string
  group: Group
  onSelect: (id: string) => void
  onBrowse: (id: string) => void
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
          <SubGroupsTab
            key={group.id}
            realm={realm}
            orgId={orgId}
            group={group}
            onSelect={onSelect}
            onBrowse={onBrowse}
          />
        </TabsContent>
      </Tabs>
    </div>
  )
}

function GroupsTable({
  realm,
  orgId,
  listing,
  selectedId,
  onSelect,
  onAddChild,
  onDelete,
}: {
  realm?: string
  orgId?: string
  listing: PagedListing
  selectedId?: string
  onSelect: (id: string) => void
  onAddChild: (group: Group) => void
  onDelete: (group: Group) => void
}) {
  const { t } = useTranslation('organization')
  const { data, isLoading } = useGroups({
    realm,
    orgId,
    query: listing.apiQuery as GroupsQuery,
  })
  const groups = useMemo(() => data?.data ?? [], [data])
  const parentIds = useMemo(
    () => groups.flatMap((group) => (group.parent_group_id ? [group.parent_group_id] : [])),
    [groups]
  )
  const { groups: parents } = useGroupsByIds({ realm, orgId, ids: parentIds })
  const parentNames = useMemo(
    () => new Map(parents.map((parent) => [parent.id, parent.name])),
    [parents]
  )
  const narrowed = Object.values(listing.state.filters).some(Boolean)

  const filterFields: FilterField[] = [
    { kind: 'text', key: 'name', label: t('groups.list.filter_fields.name') },
    { kind: 'text', key: 'description', label: t('groups.list.filter_fields.description') },
    {
      kind: 'relation',
      key: 'parent_group_id',
      label: t('groups.list.filter_fields.parent'),
      relation: groupRelationSource,
    },
    { kind: 'boolean', key: 'is_root', label: t('groups.list.filter_fields.is_root') },
  ]

  const parentLabel = (group: Group) =>
    group.parent_group_id
      ? (parentNames.get(group.parent_group_id) ?? EMPTY_VALUE)
      : t('groups.list.top_level')

  const columns: Column<Group>[] = [
    {
      key: 'name',
      header: t('groups.list.columns.name'),
      render: (group) => (
        <button
          type='button'
          className={`truncate text-left font-medium ${
            selectedId === group.id ? 'text-primary' : 'hover:text-primary'
          }`}
          onClick={() => onSelect(group.id)}
        >
          {group.name}
        </button>
      ),
      sortKey: 'name',
    },
    {
      key: 'description',
      header: t('groups.list.columns.description'),
      render: (group) => (
        <span className='text-sm text-muted-foreground'>{group.description || EMPTY_VALUE}</span>
      ),
    },
    {
      key: 'parent',
      header: t('groups.list.columns.parent'),
      render: (group) => (
        <span className='text-sm text-muted-foreground'>{parentLabel(group)}</span>
      ),
    },
    {
      key: 'created',
      header: t('groups.list.columns.created'),
      render: (group) => <span className='tnum'>{formatRelative(group.created_at)}</span>,
      sortKey: 'created_at',
    },
    {
      key: 'updated',
      header: t('groups.list.columns.updated'),
      render: (group) => <span className='tnum'>{formatRelative(group.updated_at)}</span>,
      sortKey: 'updated_at',
    },
    {
      key: 'actions',
      header: '',
      align: 'right',
      render: (group) => (
        <div className='flex justify-end gap-1'>
          <button
            type='button'
            className='text-muted-foreground hover:text-foreground'
            title={t('groups.list.show_subgroups')}
            onClick={() => listing.setFilter('parent_group_id', group.id)}
          >
            <FolderTree className='h-4 w-4' />
          </button>
          <button
            type='button'
            className='text-muted-foreground hover:text-foreground'
            title={t('groups.tree.add_child')}
            onClick={() => onAddChild(group)}
          >
            <Plus className='h-4 w-4' />
          </button>
          <button
            type='button'
            className='text-muted-foreground hover:text-destructive'
            title={t('groups.tree.delete')}
            onClick={() => onDelete(group)}
          >
            <Trash2 className='h-4 w-4' />
          </button>
        </div>
      ),
    },
  ]

  return (
    <div className='flex flex-col gap-3'>
      <div className='flex'>
        <FilterBar fields={filterFields} listing={listing} />
      </div>
      <DataView
        rows={groups}
        columns={columns}
        card={{
          title: (group) => group.name,
          subtitle: parentLabel,
          footer: (group) => group.description || EMPTY_VALUE,
        }}
        getKey={(group) => group.id}
        view={GROUPS_VIEW}
        loading={isLoading}
        sort={listing.state.sort}
        onSortChange={listing.setSort}
        emptyLabel={narrowed ? t('groups.list.no_match') : t('groups.tree.empty')}
        emptyAction={
          narrowed ? (
            <Button variant='outline' onClick={listing.clearFilters}>
              {t('groups.list.show_all')}
            </Button>
          ) : undefined
        }
      />
      {data?.metadata && (
        <PaginationBar pagination={data.metadata} onPageChange={listing.setPage} />
      )}
    </div>
  )
}

export default function PageOrganizationGroupsFeature() {
  const { realm_name, organizationId } = useParams<
    RouterParams & { organizationId: string }
  >()
  const { t } = useTranslation('organization')
  const listing = usePagedListing(GROUP_FILTER_KEYS)
  const createGroup = useCreateGroup(realm_name, organizationId)
  const deleteGroup = useDeleteGroup(realm_name, organizationId)

  const [selectedId, setSelectedId] = useState<string | undefined>()
  const [newName, setNewName] = useState('')
  const [addParent, setAddParent] = useState<Group | undefined>()
  const [childName, setChildName] = useState('')
  const [deleteTarget, setDeleteTarget] = useState<Group | undefined>()

  const { data: selected } = useGroup({
    realm: realm_name,
    orgId: organizationId,
    groupId: selectedId,
  })

  const createRoot = () => {
    if (!newName) return
    createGroup.mutate({ name: newName }, { onError: fail, onSuccess: () => setNewName('') })
  }

  const addChild = (parent: Group) => {
    setChildName('')
    setAddParent(parent)
  }

  const browse = (parentId: string) => listing.setFilter('parent_group_id', parentId)

  const submitChild = () => {
    if (!childName || !addParent) return
    createGroup.mutate(
      { name: childName, parent_group_id: addParent.id },
      {
        onError: fail,
        onSuccess: () => {
          setChildName('')
          setAddParent(undefined)
        },
      }
    )
  }

  const confirmDelete = () => {
    if (!deleteTarget) return
    const target = deleteTarget
    deleteGroup.mutate(target.id, {
      onError: fail,
      onSuccess: () => {
        setSelectedId((id) => (id === target.id ? undefined : id))
        setDeleteTarget(undefined)
      },
    })
  }

  return (
    <div className='flex flex-col gap-6'>
      <div className='flex flex-col gap-3 rounded-md border p-3'>
        <div className='flex items-center gap-2'>
          <Input
            className='max-w-sm'
            placeholder={t('groups.tree.new_placeholder')}
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && createRoot()}
          />
          <Button variant='outline' disabled={!newName} onClick={createRoot}>
            {t('groups.tree.add')}
          </Button>
        </div>
        <GroupsTable
          realm={realm_name}
          orgId={organizationId}
          listing={listing}
          selectedId={selectedId}
          onSelect={setSelectedId}
          onAddChild={addChild}
          onDelete={setDeleteTarget}
        />
      </div>

      <div className='rounded-md border p-4'>
        {selected ? (
          <GroupDetail
            realm={realm_name}
            orgId={organizationId}
            group={selected}
            onSelect={setSelectedId}
            onBrowse={browse}
          />
        ) : (
          <p className='text-sm text-muted-foreground'>{t('groups.no_selection')}</p>
        )}
      </div>

      <Dialog
        open={addParent !== undefined}
        onOpenChange={(open) => !open && setAddParent(undefined)}
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
            <Button variant='ghost' onClick={() => setAddParent(undefined)}>
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
