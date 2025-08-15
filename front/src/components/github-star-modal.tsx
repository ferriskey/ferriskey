import { useEffect, useState } from 'react'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from './ui/dialog'
import { Button } from './ui/button'
import { GitBranch, Heart, Star } from 'lucide-react'

const STORAGE_KEY = 'ferriskey-github-star-modal-dismissed'

export default function GithubStarModal() {
  const [isOpen, setIsOpen] = useState(false)

  useEffect(() => {
    const hasSeenModal = localStorage.getItem(STORAGE_KEY)

    if (!hasSeenModal) {
      const timer = setTimeout(() => {
        setIsOpen(true)
      }, 2000)

      return () => clearTimeout(timer)
    }
  }, [])

  const handleDismiss = () => {
    localStorage.setItem(STORAGE_KEY, 'true')
    setIsOpen(false)
  }

  const handleStarProject = () => {
    localStorage.setItem(STORAGE_KEY, 'true')
    window.open('https://github.com/ferriskey/ferriskey', '_blank')
    setIsOpen(false)
  }

  return (
    <Dialog
      open={isOpen}
      onOpenChange={(open) => {
        if (!open) {
          // Si la modal se ferme (peu importe comment), marquer comme vue
          localStorage.setItem(STORAGE_KEY, 'true')
          setIsOpen(false)
        }
      }}
    >
      <DialogContent className='max-w-md mx-auto'>
        <div className='flex items-center justify-between mb-4'>
          <div className='flex items-center gap-2'>
            <div className='p-2 bg-gradient-to-br from-orange-400 to-red-500 rounded-lg'>
              <Heart className='h-5 w-5 text-white' />
            </div>
            <DialogTitle className='text-xl font-semibold'>Support FerrisKey! 🦀</DialogTitle>
          </div>
        </div>

        <div className='space-y-6'>
          {/* Hero Image/Icon */}
          <div className='flex justify-center'>
            <div className='relative'>
              <div className='w-20 h-20 bg-gradient-to-br from-blue-400 via-purple-500 to-pink-500 rounded-full flex items-center justify-center animate-pulse'>
                <GitBranch className='h-10 w-10 text-white' />
              </div>
              <div className='absolute -top-1 -right-1 w-6 h-6 bg-yellow-400 rounded-full flex items-center justify-center animate-bounce'>
                <Star className='h-3 w-3 text-yellow-800' fill='currentColor' />
              </div>
            </div>
          </div>

          <DialogDescription className='text-center space-y-3'>
            <p className='text-lg font-medium text-foreground'>Welcome to FerrisKey! 👋</p>
            <p className='text-muted-foreground'>
              FerrisKey is an open-source project built with passion. If you find this tool useful,
              support us by adding a star on GitHub!
            </p>
            <div className='flex items-center justify-center gap-4 py-2'>
              <div className='flex items-center gap-1 text-sm text-muted-foreground'>
                <Star className='h-4 w-4' />
                <span>Free & Open Source</span>
              </div>
              <div className='flex items-center gap-1 text-sm text-muted-foreground'>
                <Heart className='h-4 w-4' />
                <span>Made with ❤️</span>
              </div>
            </div>
          </DialogDescription>

          {/* Actions */}
          <div className='flex flex-col gap-3'>
            <Button
              onClick={handleStarProject}
              className='w-full bg-gradient-to-r from-orange-500 to-red-500 hover:from-orange-600 hover:to-red-600 text-white font-medium'
              size='lg'
            >
              <Star className='h-4 w-4 mr-2' />⭐ Star on GitHub
            </Button>

            <Button
              variant='ghost'
              onClick={handleDismiss}
              className='w-full text-muted-foreground hover:text-foreground'
            >
              Maybe later
            </Button>
          </div>

          {/* Footer note */}
          <p className='text-xs text-center text-muted-foreground border-t pt-4'>
            This popup will only show once. Thank you for your support! 🙏
          </p>
        </div>
      </DialogContent>
    </Dialog>
  )
}
