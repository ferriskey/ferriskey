import type { Active } from '@dnd-kit/core'
import { useEffect, useState } from 'react'
import { createPortal } from 'react-dom'
import { useBuilderContext } from '../context'

interface BuilderDragOverlayProps {
  activeItem: Active | null
}

/**
 * Custom drag chip rendered alongside (not via) `@dnd-kit`'s `DragOverlay`.
 *
 * Why not `DragOverlay`: dnd-kit's `PointerSensor` activates with
 * `setPointerCapture` on the source element. When the source lives inside
 * the canvas iframe, that capture pins all subsequent `pointermove` events
 * to the iframe's document — even though the canvas sets the iframe's
 * `pointer-events: none` so the parent's collision logic can still reason
 * about the drag. The visible side effect through `DragOverlay` is that the
 * overlay's transform is computed in iframe-local coords and rendered in the
 * parent document at that same pixel value, so the chip floats off by the
 * iframe's offset within the parent viewport (and by the iframe's CSS scale
 * when previewing tablet/mobile sizes).
 *
 * Bypass: render the chip ourselves as a `position: fixed` portal into
 * `document.body`, and update its position from BOTH the parent `window`
 * (covers library drags) AND every iframe's `contentWindow` (covers canvas
 * drags). Iframe-local pointer coordinates are translated into parent-window
 * coords using the iframe's bounding rect, which already accounts for any
 * CSS scale applied to the iframe element.
 */
export function BuilderDragOverlay({ activeItem }: BuilderDragOverlayProps) {
  const { adapter } = useBuilderContext()
  const [pointer, setPointer] = useState<{ x: number; y: number } | null>(null)

  // Only mount listeners while a drag is in progress so we don't pay for
  // pointer tracking when the user is just navigating the editor. The chip
  // is gated on `!activeItem` below, so a leftover `pointer` value from the
  // previous drag is harmless — we skip resetting it here to avoid a
  // synchronous `setState` inside an effect (caught by the React Compiler
  // lint rule).
  useEffect(() => {
    if (!activeItem) return

    const onParentMove = (e: PointerEvent) => {
      setPointer({ x: e.clientX, y: e.clientY })
    }
    window.addEventListener('pointermove', onParentMove, { passive: true })

    // For canvas drags, `setPointerCapture` on the source pins moves to the
    // iframe's document. Attach a listener to every iframe in the page; for
    // each event, translate the iframe-local coords into parent-window coords
    // using the iframe element's bounding rect (which already encodes any
    // CSS scale via `width / clientWidth`).
    const iframeCleanups: Array<() => void> = []
    document.querySelectorAll('iframe').forEach((iframe) => {
      const win = iframe.contentWindow
      if (!win) return
      const onIframeMove = (e: PointerEvent) => {
        const rect = iframe.getBoundingClientRect()
        const innerW = iframe.clientWidth || rect.width
        const innerH = iframe.clientHeight || rect.height
        const scaleX = innerW ? rect.width / innerW : 1
        const scaleY = innerH ? rect.height / innerH : 1
        setPointer({
          x: e.clientX * scaleX + rect.left,
          y: e.clientY * scaleY + rect.top,
        })
      }
      try {
        win.addEventListener('pointermove', onIframeMove, { passive: true })
        iframeCleanups.push(() => {
          try {
            win.removeEventListener('pointermove', onIframeMove)
          } catch {
            // Iframe may have been unmounted before cleanup runs.
          }
        })
      } catch {
        // Cross-origin iframes throw on access; skip them — there's nothing
        // useful we can do, and they aren't part of the builder canvas.
      }
    })

    return () => {
      window.removeEventListener('pointermove', onParentMove)
      iframeCleanups.forEach((c) => c())
    }
  }, [activeItem])

  if (!activeItem || !pointer) return null

  const data = activeItem.data.current
  const type =
    data?.source === 'library' ? data.type : data?.source === 'canvas' ? data.node?.type : null

  if (!type) return null

  const def = adapter.components.find((c) => c.type === type)

  return createPortal(
    <div
      style={{
        position: 'fixed',
        // `pointer-events: none` so the chip never intercepts the move/up
        // events dnd-kit needs to keep tracking the drag.
        pointerEvents: 'none',
        // Sit above every editor surface (the iframe, the side panels) so the
        // chip is always visible — but stay under most modal portals.
        zIndex: 9999,
        // Tiny nudge below the cursor so the chip's top-left lands a couple
        // of pixels off the hot-spot, matching the "next to cursor" feel of
        // native browser drag images.
        left: pointer.x + 4,
        top: pointer.y + 4,
      }}
    >
      <div className='flex items-center gap-2 rounded-md border border-primary bg-card px-3 py-2 text-sm shadow-lg'>
        {def?.icon && (
          <span className='flex h-4 w-4 items-center justify-center text-muted-foreground'>
            {def.icon}
          </span>
        )}
        <span>{def?.label ?? type}</span>
      </div>
    </div>,
    document.body
  )
}
