import { useEffect, useMemo, useRef, useState } from 'react'

interface FitOptions {
  /** Logical width of the iframe content (e.g. 1280 for desktop). */
  width: number
  /** Logical height of the iframe content (e.g. 800 for desktop). */
  height: number
  /** Padding (px) reserved on every side of the container. */
  padding?: number
}

/**
 * Computes the `transform: scale(...)` factor needed to fit an iframe of
 * `width × height` inside its measured container. The container ref must
 * point to the scroll-area that hosts the iframe; the hook observes its
 * size and recomputes when either dimension changes.
 *
 * Returned scale is clamped to `1` so iframes smaller than the container
 * render at their natural size instead of being magnified.
 */
export function useIframeFit({ width, height, padding = 0 }: FitOptions) {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const [size, setSize] = useState<{ width: number; height: number } | null>(null)

  useEffect(() => {
    const el = containerRef.current
    if (!el) return
    const publish = () => {
      const rect = el.getBoundingClientRect()
      setSize({ width: rect.width, height: rect.height })
    }
    publish()
    const ro = new ResizeObserver(publish)
    ro.observe(el)
    return () => ro.disconnect()
  }, [])

  const scale = useMemo(() => {
    if (!size) return 1
    const availW = Math.max(0, size.width - padding * 2)
    const availH = Math.max(0, size.height - padding * 2)
    if (availW === 0 || availH === 0) return 1
    return Math.min(availW / width, availH / height, 1)
  }, [size, width, height, padding])

  return { containerRef, scale }
}
