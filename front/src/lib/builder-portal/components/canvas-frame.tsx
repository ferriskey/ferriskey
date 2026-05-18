import { useDndMonitor } from '@dnd-kit/core'
import { useEffect, useRef, type CSSProperties, type ReactNode } from 'react'
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

  useDndMonitor({
    onDragStart: () => setIframeInteractive(false),
    onDragEnd: () => setIframeInteractive(true),
    onDragCancel: () => setIframeInteractive(true),
  })

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
          <FrameBody ctx={ctx} cssVars={cssVars}>
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
  children,
}: {
  ctx: FrameContextProps
  cssVars: CSSProperties
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
