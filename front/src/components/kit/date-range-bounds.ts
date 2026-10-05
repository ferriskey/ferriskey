const DAY_PATTERN = /^(\d{4})-(\d{2})-(\d{2})$/
const DAY_MS = 86_400_000

function parseDay(day: string): number | null {
  const match = DAY_PATTERN.exec(day)
  if (!match) return null
  const [, year, month, date] = match
  const time = Date.UTC(Number(year), Number(month) - 1, Number(date))
  const parsed = new Date(time)
  if (
    parsed.getUTCFullYear() !== Number(year) ||
    parsed.getUTCMonth() !== Number(month) - 1 ||
    parsed.getUTCDate() !== Number(date)
  ) {
    return null
  }
  return time
}

function formatInstant(time: number): string {
  return `${new Date(time).toISOString().slice(0, 19)}Z`
}

function formatDay(time: number): string {
  return new Date(time).toISOString().slice(0, 10)
}

function parseInstant(value: string): number | null {
  if (value === '') return null
  const time = Date.parse(value)
  return Number.isNaN(time) ? null : time
}

export function toRangeBounds(fromDay: string, toDay: string): { from: string; to: string } {
  const from = parseDay(fromDay)
  const to = parseDay(toDay)
  return {
    from: from === null ? '' : formatInstant(from),
    to: to === null ? '' : formatInstant(to + DAY_MS),
  }
}

export function fromRangeBounds(from: string, to: string): { fromDay: string; toDay: string } {
  const start = parseInstant(from)
  const end = parseInstant(to)
  return {
    fromDay: start === null ? '' : formatDay(start),
    toDay: end === null ? '' : formatDay(end - 1),
  }
}

export function withRangeDay(
  current: { from: string; to: string },
  side: 'from' | 'to',
  day: string,
): { from: string; to: string } {
  return side === 'from'
    ? { from: toRangeBounds(day, '').from, to: current.to }
    : { from: current.from, to: toRangeBounds('', day).to }
}

export interface CalendarRange {
  from: Date | undefined
  to?: Date
}

function dayToDate(day: string): Date | undefined {
  const match = DAY_PATTERN.exec(day)
  if (!match || parseDay(day) === null) return undefined
  const [, year, month, date] = match
  return new Date(Number(year), Number(month) - 1, Number(date))
}

function dateToDay(date: Date | undefined): string {
  if (!date) return ''
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${date.getFullYear()}-${month}-${day}`
}

export function calendarRangeFromBounds(from: string, to: string): CalendarRange | undefined {
  const { fromDay, toDay } = fromRangeBounds(from, to)
  const start = dayToDate(fromDay)
  const end = dayToDate(toDay)
  if (!start && !end) return undefined
  return { from: start, to: end }
}

export function boundsFromCalendarRange(
  current: { from: string; to: string },
  range: CalendarRange | undefined,
): { from: string; to: string } {
  const shown = fromRangeBounds(current.from, current.to)
  const fromDay = dateToDay(range?.from)
  const toDay = dateToDay(range?.to)
  const withFrom = fromDay === shown.fromDay ? current : withRangeDay(current, 'from', fromDay)
  return toDay === shown.toDay ? withFrom : withRangeDay(withFrom, 'to', toDay)
}
