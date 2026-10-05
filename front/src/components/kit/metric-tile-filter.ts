export type TileFilter = Record<string, string>

export function tileFilterKeys(tiles: readonly { filter?: TileFilter }[]): string[] {
  return [...new Set(tiles.flatMap((tile) => Object.keys(tile.filter ?? {})))]
}

export function tileFilterPatch(
  tiles: readonly { filter?: TileFilter }[],
  filter: TileFilter,
): Record<string, string> {
  return { ...Object.fromEntries(tileFilterKeys(tiles).map((key) => [key, ''])), ...filter }
}

export function isTileSelected(
  tiles: readonly { filter?: TileFilter }[],
  filter: TileFilter,
  filters: Record<string, string>,
): boolean {
  return Object.entries(tileFilterPatch(tiles, filter)).every(
    ([key, value]) => (filters[key] ?? '') === value,
  )
}
