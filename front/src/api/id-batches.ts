export const ID_BATCH = 100

export function idBatches(ids: readonly string[]): string[] {
  const unique = [...new Set(ids)].sort()
  const batches: string[] = []
  for (let start = 0; start < unique.length; start += ID_BATCH) {
    batches.push(unique.slice(start, start + ID_BATCH).join(','))
  }
  return batches
}
