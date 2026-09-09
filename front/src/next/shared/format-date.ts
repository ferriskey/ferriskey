const LOCALE = 'en-GB'

const dateOnly = new Intl.DateTimeFormat(LOCALE, {
  day: 'numeric',
  month: 'short',
  year: 'numeric',
})

const dateTime = new Intl.DateTimeFormat(LOCALE, {
  day: 'numeric',
  month: 'short',
  year: 'numeric',
  hour: '2-digit',
  minute: '2-digit',
  hour12: false,
})

const timestamp = new Intl.DateTimeFormat(LOCALE, {
  day: 'numeric',
  month: 'short',
  year: 'numeric',
  hour: '2-digit',
  minute: '2-digit',
  second: '2-digit',
  hour12: false,
})

const timeOnly = new Intl.DateTimeFormat(LOCALE, {
  hour: '2-digit',
  minute: '2-digit',
  second: '2-digit',
  hour12: false,
})

const parse = (iso: string) => {
  const date = new Date(iso)
  return Number.isNaN(date.getTime()) ? null : date
}

export const formatDate = (iso: string) => {
  const date = parse(iso)
  return date ? dateOnly.format(date) : iso
}

export const formatDateTime = (iso: string) => {
  const date = parse(iso)
  return date ? dateTime.format(date) : iso
}

export const formatTimestamp = (iso: string) => {
  const date = parse(iso)
  return date ? timestamp.format(date) : iso
}

export const formatTime = (iso: string) => {
  const date = parse(iso)
  return date ? timeOnly.format(date) : iso
}

const MINUTE = 60_000
const HOUR = 60 * MINUTE
const DAY = 24 * HOUR

export const formatRelative = (iso: string, now: number = Date.now()) => {
  const date = parse(iso)
  if (!date) return iso

  const elapsed = now - date.getTime()
  if (elapsed < 0) return dateTime.format(date)
  if (elapsed < MINUTE) return 'just now'
  if (elapsed < HOUR) return `${Math.floor(elapsed / MINUTE)} min ago`
  if (elapsed < DAY) return `${Math.floor(elapsed / HOUR)} h ago`
  if (elapsed < 7 * DAY) return `${Math.floor(elapsed / DAY)} d ago`
  return dateOnly.format(date)
}
