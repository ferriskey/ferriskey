import { useDndMonitor } from '@dnd-kit/core'
import { useCallback, useEffect, useRef, type CSSProperties, type ReactNode } from 'react'
import Frame, { FrameContextConsumer, type FrameContextProps } from 'react-frame-component'

interface Props {
  /** Width applied to the iframe element (e.g. 768 for tablet preview). */
  width: number | string
  /**
   * Height applied to the iframe element — drives what `100vh` resolves to
   * inside the canvas. Defaults to 720 (a reasonable desktop minimum).
   */
  height?: number | string
  /** Inline CSS variables injected into the iframe body (theme tokens). */
  cssVars: CSSProperties
  /**
   * Optional CSS string (typically `generateBreakpointCss(tree)`) injected
   * into the iframe head. The iframe's real width drives which `@media`
   * rules fire, matching what the live portal will do.
   */
  responsiveCss?: string
  /** Rendered into the iframe document via React portal. */
  children: ReactNode
  /**
   * Reports the iframe element's bounding rect in the parent viewport so the
   * builder shell can translate pointer coordinates into iframe-local space
   * for dnd-kit collision detection. Called whenever the iframe resizes.
   */
  onRectChange?: (rect: DOMRect | null) => void
}

/**
 * Renders the portal builder canvas inside a same-origin iframe.
 *
 * Why an iframe: viewport-relative units (`vw`, `vh`, `dvw`, `dvh`, …),
 * `position: fixed`, and `@media (max-width: …)` queries resolve against
 * the iframe's own viewport, so the preview is a faithful WYSIWYG of the
 * device size selected in the toolbar.
 *
 * Drag-and-drop interop: while a drag is in progress we set
 * `pointer-events: none` on the iframe so the parent window keeps receiving
 * `pointermove` / `pointerup`. Droppables rendered inside the iframe still
 * register with the parent's DndContext (React context survives the portal).
 * Collision detection sees their iframe-local rects — the parent shell
 * compensates by offsetting pointer coordinates by the iframe rect (see
 * `onRectChange` above + the custom collision detection in
 * `BuilderShell`).
 */
export function CanvasFrame({
  width,
  height = 720,
  cssVars,
  responsiveCss,
  children,
  onRectChange,
}: Props) {
  const iframeRef = useRef<HTMLIFrameElement | null>(null)

  // Toggle pointer-events imperatively so the swap happens INSIDE the dnd-kit
  // callback (synchronously, no React state batching). If we waited for a
  // state update + render, the cursor could already cross the iframe boundary
  // with pointer-events: auto and the iframe would steal the move events from
  // the parent's PointerSensor, breaking the drag.
  const setIframeInteractive = (interactive: boolean) => {
    const el = iframeRef.current
    if (el) el.style.pointerEvents = interactive ? 'auto' : 'none'
  }

  // Auto-scroll the iframe's content while dragging: dnd-kit's built-in
  // auto-scroll only knows about scrollable parents in the *parent*
  // document, but the page tree lives inside the iframe and the iframe
  // itself is sized to a fixed viewport (e.g. 1600×720 desktop). Without
  // this hook, the admin can't reach any block that's below the fold —
  // the mouse wheel doesn't fire on a `pointer-events: none` iframe
  // during the drag.
  const autoScrollRafRef = useRef<number | null>(null)
  const autoScrollDirRef = useRef<-1 | 0 | 1>(0)
  const startAutoScroll = useCallback(() => {
    if (autoScrollRafRef.current !== null) return
    const tick = () => {
      const dir = autoScrollDirRef.current
      const win = iframeRef.current?.contentWindow
      if (dir !== 0 && win) {
        // 12px per frame ≈ 720px/s at 60fps — fast enough to traverse a
        // tall page in a couple of seconds without flying past targets.
        win.scrollBy(0, dir * 12)
      }
      autoScrollRafRef.current = requestAnimationFrame(tick)
    }
    autoScrollRafRef.current = requestAnimationFrame(tick)
  }, [])
  const stopAutoScroll = useCallback(() => {
    if (autoScrollRafRef.current !== null) {
      cancelAnimationFrame(autoScrollRafRef.current)
      autoScrollRafRef.current = null
    }
    autoScrollDirRef.current = 0
  }, [])

  useDndMonitor({
    onDragStart: () => {
      setIframeInteractive(false)
      startAutoScroll()
    },
    // `onDragMove` fires with the current pointer position in parent-window
    // coords (after `setPointerCapture` redirects events to the iframe, the
    // event view is the iframe but the coordinates have been normalised by
    // dnd-kit's sensor before reaching us — see the comment in
    // `drag-overlay.tsx` for the gnarly details). We compute the cursor's
    // distance to the iframe's top/bottom edges and set a scroll direction
    // that the RAF loop above consumes. Threshold is generous (60px) so the
    // admin doesn't have to be pixel-perfect to trigger scroll.
    onDragMove: (event) => {
      const el = iframeRef.current
      if (!el) return
      const rect = el.getBoundingClientRect()
      // `event.delta` is relative to drag start. We need the absolute
      // pointer Y — dnd-kit doesn't expose it directly here, so derive it
      // from the active rect's translated position (the centre of the
      // dragged block tracks the cursor's vertical movement once dnd-kit
      // is past the activation threshold).
      const translated = event.active.rect.current.translated
      if (!translated) return
      const pointerY = translated.top + translated.height / 2
      const threshold = 60
      if (pointerY < rect.top + threshold) {
        autoScrollDirRef.current = -1
      } else if (pointerY > rect.bottom - threshold) {
        autoScrollDirRef.current = 1
      } else {
        autoScrollDirRef.current = 0
      }
    },
    onDragEnd: () => {
      setIframeInteractive(true)
      stopAutoScroll()
    },
    onDragCancel: () => {
      setIframeInteractive(true)
      stopAutoScroll()
    },
  })

  useEffect(() => () => stopAutoScroll(), [stopAutoScroll])

  // Re-publish the iframe's bounding rect whenever its size changes or the
  // parent window resizes — collision detection needs the up-to-date offset.
  useEffect(() => {
    const el = iframeRef.current
    if (!el) return

    const publish = () => onRectChange?.(el.getBoundingClientRect())
    publish()

    const ro = new ResizeObserver(publish)
    ro.observe(el)
    window.addEventListener('scroll', publish, true)
    window.addEventListener('resize', publish)

    return () => {
      ro.disconnect()
      window.removeEventListener('scroll', publish, true)
      window.removeEventListener('resize', publish)
    }
  }, [onRectChange, width])

  return (
    <Frame
      ref={iframeRef}
      style={{
        width,
        // Concrete height so `100vh` inside the iframe resolves to the
        // selected device's viewport instead of the iframe's default 150px.
        height,
        border: 'none',
        display: 'block',
      }}
      initialContent={
        '<!DOCTYPE html><html><head><base target="_parent" /></head><body style="margin:0;font-family:inherit"><div class="frame-root"></div></body></html>'
      }
    >
      <FrameContextConsumer>
        {(ctx: FrameContextProps) => (
          <FrameBody ctx={ctx} cssVars={cssVars} responsiveCss={responsiveCss}>
            {children}
          </FrameBody>
        )}
      </FrameContextConsumer>
    </Frame>
  )
}

/**
 * Applies the theme CSS variables to the iframe's <body>, ensures the body
 * has sane defaults, and keeps the iframe head in sync with the parent
 * document's stylesheets (so Tailwind / Vite HMR updates flow through).
 */
function FrameBody({
  ctx,
  cssVars,
  responsiveCss,
  children,
}: {
  ctx: FrameContextProps
  cssVars: CSSProperties
  responsiveCss?: string
  children: ReactNode
}) {
  useEffect(() => {
    const doc = ctx.document
    if (!doc) return
    const body = doc.body
    body.style.setProperty('margin', '0')
    body.style.setProperty('font-family', 'inherit')
    for (const [k, v] of Object.entries(cssVars)) {
      if (typeof v === 'string') body.style.setProperty(k, v)
    }
  }, [ctx.document, cssVars])

  // Sync the responsive `@media` rules into a dedicated <style> tag in the
  // iframe head. Re-runs whenever the user edits a breakpoint override.
  // We re-append on every update so the style stays AFTER the cloned parent
  // stylesheets (which the sync useEffect re-appends later); equal-specificity
  // !important rules from later-in-cascade sheets would otherwise win.
  useEffect(() => {
    const doc = ctx.document
    if (!doc) return
    const head = doc.head
    let style = head.querySelector<HTMLStyleElement>('style[data-fk-responsive]')
    if (!style) {
      style = doc.createElement('style')
      style.dataset.fkResponsive = '1'
    }
    style.textContent = responsiveCss ?? ''
    head.appendChild(style)
  }, [ctx.document, responsiveCss])

  // Mirror every stylesheet (link + style) from the parent into the iframe
  // head, and re-sync whenever the parent's head changes (Vite injects CSS
  // dynamically during dev / HMR).
  useEffect(() => {
    const doc = ctx.document
    if (!doc) return
    const parentHead = document.head
    const frameHead = doc.head

    const sync = () => {
      // Wipe any cloned nodes we previously injected, keep <base> + others
      // that react-frame-component itself adds.
      frameHead.querySelectorAll('[data-fk-clone]').forEach((n) => n.remove())

      parentHead.querySelectorAll('link[rel="stylesheet"]').forEach((node) => {
        const href = (node as HTMLLinkElement).href
        if (!href) return
        const link = doc.createElement('link')
        link.rel = 'stylesheet'
        link.href = href
        link.dataset.fkClone = '1'
        frameHead.appendChild(link)
      })

      parentHead.querySelectorAll('style').forEach((node) => {
        const style = doc.createElement('style')
        style.textContent = (node as HTMLStyleElement).textContent ?? ''
        style.dataset.fkClone = '1'
        frameHead.appendChild(style)
      })
    }

    sync()
    const observer = new MutationObserver(sync)
    observer.observe(parentHead, { childList: true, subtree: true, characterData: true })
    return () => observer.disconnect()
  }, [ctx.document])

  return <>{children}</>
}
