export interface StyleTokens {
  page: {
    padding: string
    maxWidth: string
    sectionGap: string
    blockGap: string
  }

  header: {
    title: string
    spacing: string
  }

  table: {
    cellPadding: string
    headerPadding: string
    text: string
    headerText: string
    showFooterAggregates: boolean
    sortable: boolean
    selectable: boolean
  }

  card: {
    padding: string
    gap: string
    columns: string
    showFlows: boolean
  }

  surface: {
    panel: string
    divider: string
  }

  toolbar: {
    showQuerySyntax: boolean
    showViewToggle: boolean
  }
}

export const tokens: StyleTokens = {
  page: {
    padding: 'px-5 py-4',
    maxWidth: 'max-w-[1600px]',
    sectionGap: 'space-y-3',
    blockGap: 'space-y-6',
  },
  header: {
    title: 'text-lg font-semibold tracking-tight',
    spacing: 'pb-3',
  },
  table: {
    cellPadding: 'px-2.5 py-2',
    headerPadding: 'px-2.5 py-1.5',
    text: 'text-[13px]',
    headerText: 'text-[11px] font-medium uppercase tracking-wide text-neutral-400',
    showFooterAggregates: true,
    sortable: true,
    selectable: true,
  },
  card: {
    padding: 'p-3',
    gap: 'gap-2',
    columns: 'sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4',
    showFlows: true,
  },
  surface: {
    panel: 'rounded-sm border border-fk-line bg-white',
    divider: 'divide-y divide-fk-line-soft',
  },
  toolbar: {
    showQuerySyntax: true,
    showViewToggle: true,
  },
}
