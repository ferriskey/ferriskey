import { useEditor, EditorContent } from '@tiptap/react'
import StarterKit from '@tiptap/starter-kit'
import Link from '@tiptap/extension-link'
import Underline from '@tiptap/extension-underline'
import { useEffect } from 'react'
import {
  Bold,
  Italic,
  Underline as UnderlineIcon,
  Link as LinkIcon,
  List,
  ListOrdered,
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
      StarterKit,
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
      <div className='absolute bottom-full left-4 mb-1 z-20 flex flex-wrap items-center gap-0.5 rounded border border-border bg-background px-1 py-0.5 shadow-md'>
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
            <select
              className='rounded bg-background px-1 py-0.5 text-xs border border-border'
              value=''
              onChange={(e) => {
                if (e.target.value) insertVariable(e.target.value)
              }}
            >
              <option value=''>+ Variable</option>
              {variables.map((v) => (
                <option key={v.name} value={v.name}>
                  {`{{${v.name}}}`}
                </option>
              ))}
            </select>
          </>
        )}
      </div>
      <EditorContent
        editor={editor}
        className='prose prose-sm max-w-none [&_.ProseMirror]:outline-none [&_.ProseMirror]:min-h-[1em]'
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
