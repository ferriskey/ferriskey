import { useEditor, EditorContent } from '@tiptap/react'
import StarterKit from '@tiptap/starter-kit'
import Link from '@tiptap/extension-link'
import Underline from '@tiptap/extension-underline'
import { useEffect, useRef, useState } from 'react'
import {
  Bold,
  Italic,
  Underline as UnderlineIcon,
  Link as LinkIcon,
  List,
  ListOrdered,
  Braces,
} from 'lucide-react'

interface TemplateVariable {
  name: string
  description: string
}

interface InlineTextEditorProps {
  content: string
  onChange: (html: string) => void
  variables?: TemplateVariable[]
  className?: string
}

export function InlineTextEditor({
  content,
  onChange,
  variables,
  className,
}: InlineTextEditorProps) {
  const editor = useEditor({
    extensions: [
      StarterKit.configure({
        heading: false,
      }),
      Underline,
      Link.configure({ openOnClick: false }),
    ],
    content,
    onUpdate: ({ editor: e }) => {
      onChange(e.getHTML())
    },
  })

  useEffect(() => {
    if (editor && content !== editor.getHTML()) {
      editor.commands.setContent(content, { emitUpdate: false })
    }
  }, [content, editor])

  if (!editor) return null

  const insertVariable = (varName: string) => {
    editor.chain().focus().insertContent(`{{${varName}}}`).run()
  }

  return (
    <div
      className={className}
      onPointerDown={(e) => e.stopPropagation()}
      onClick={(e) => e.stopPropagation()}
    >
      <div className='absolute bottom-full left-4 mb-1 z-50 flex items-center gap-0.5 rounded border border-border bg-background px-1 py-0.5 shadow-md whitespace-nowrap'>
        <ToolbarButton
          active={editor.isActive('bold')}
          onClick={() => editor.chain().focus().toggleBold().run()}
        >
          <Bold size={12} />
        </ToolbarButton>
        <ToolbarButton
          active={editor.isActive('italic')}
          onClick={() => editor.chain().focus().toggleItalic().run()}
        >
          <Italic size={12} />
        </ToolbarButton>
        <ToolbarButton
          active={editor.isActive('underline')}
          onClick={() => editor.chain().focus().toggleUnderline().run()}
        >
          <UnderlineIcon size={12} />
        </ToolbarButton>
        <ToolbarButton
          active={editor.isActive('link')}
          onClick={() => {
            const url = window.prompt('URL')
            if (url) {
              editor.chain().focus().setLink({ href: url }).run()
            }
          }}
        >
          <LinkIcon size={12} />
        </ToolbarButton>
        <div className='mx-0.5 h-4 w-px bg-border' />
        <ToolbarButton
          active={editor.isActive('bulletList')}
          onClick={() => editor.chain().focus().toggleBulletList().run()}
        >
          <List size={12} />
        </ToolbarButton>
        <ToolbarButton
          active={editor.isActive('orderedList')}
          onClick={() => editor.chain().focus().toggleOrderedList().run()}
        >
          <ListOrdered size={12} />
        </ToolbarButton>

        {variables && variables.length > 0 && (
          <>
            <div className='mx-0.5 h-4 w-px bg-border' />
            <VariableDropdown variables={variables} onInsert={insertVariable} />
          </>
        )}
      </div>
      <EditorContent
        editor={editor}
        className='max-w-none [&_.ProseMirror]:outline-none [&_.ProseMirror]:min-h-[1em] [&_.ProseMirror_p]:m-0 [&_.ProseMirror_h1]:m-0 [&_.ProseMirror_h2]:m-0 [&_.ProseMirror_h3]:m-0 [&_.ProseMirror_ul]:m-0 [&_.ProseMirror_ol]:m-0'
      />
    </div>
  )
}

function ToolbarButton({
  active,
  onClick,
  children,
}: {
  active: boolean
  onClick: () => void
  children: React.ReactNode
}) {
  return (
    <button
      type='button'
      className={`rounded p-1 transition-colors ${
        active
          ? 'bg-primary/10 text-primary'
          : 'text-muted-foreground hover:bg-muted'
      }`}
      onClick={onClick}
    >
      {children}
    </button>
  )
}

function VariableDropdown({
  variables,
  onInsert,
}: {
  variables: TemplateVariable[]
  onInsert: (varName: string) => void
}) {
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!open) return
    function handleClickOutside(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [open])

  return (
    <div ref={ref} className='relative'>
      <button
        type='button'
        className={`rounded p-1 transition-colors ${
          open
            ? 'bg-primary/10 text-primary'
            : 'text-muted-foreground hover:bg-muted'
        }`}
        onClick={() => setOpen(!open)}
      >
        <Braces size={12} />
      </button>
      {open && (
        <div className='absolute top-full left-0 mt-1 min-w-[160px] rounded border border-border bg-background py-1 shadow-lg z-50'>
          {variables.map((v) => (
            <button
              key={v.name}
              type='button'
              className='flex w-full items-center gap-2 px-2 py-1 text-left text-xs hover:bg-muted transition-colors'
              onClick={() => {
                onInsert(v.name)
                setOpen(false)
              }}
            >
              <span className='font-mono text-primary'>{`{{${v.name}}}`}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
