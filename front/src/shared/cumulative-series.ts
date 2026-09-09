const DEFAULT_WINDOW_DAYS = 30
const DAY = 24 * 60 * 60 * 1000

function windowDays(days: number) {
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  return Array.from({ length: days }, (_, i) => {
    const day = new Date(today)
    day.setDate(today.getDate() - (days - 1 - i))
    return day.getTime()
  })
}

export function cumulativeSeries(
  createdAt: Array<string | null | undefined>,
  days: number = DEFAULT_WINDOW_DAYS
) {
  const stamps = createdAt
    .map((value) => (value ? new Date(value).getTime() : Number.NaN))
    .filter((time) => !Number.isNaN(time))

  if (stamps.length === 0) return undefined

  return windowDays(days).map((day) => stamps.filter((time) => time < day + DAY).length)
}
