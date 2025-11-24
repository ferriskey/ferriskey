import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { User } from 'lucide-react'

export interface FlaggedUsersProps {
  users: {
    name: string,
    email: string
    avatar: string
    reason: string
    count: number
    risk: string
  }[]
}

export function FlaggedUsers({ users }: FlaggedUsersProps) {
  return (
    <div>
      <Card>
        <CardHeader className='pb-4 blur-[4px]'>
          <CardTitle className='flex items-center gap-2'>
            <User className='h-4 w-4 text-amber-500' />
            Flagged Users
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className='space-y-3 blur-[4px]'>
            {users.map((user, index) => (
              <div key={index} className='flex items-center space-x-3 border rounded-md p-3 hover:shadow-md hover:cursor-pointer shadow-primary/10 transition-all'>
                <Avatar className='h-8 w-8'>
                  <AvatarFallback className='bg-primary/10 text-primary font-medium text-xs'>
                    {user.avatar}
                  </AvatarFallback>
                </Avatar>
                <div className='flex-1 min-w-0'>
                  <p className='font-medium truncate'>
                    {user.name}
                  </p>
                  <p className='text-sm text-muted-foreground truncate'>
                    {user.email}
                  </p>
                  <p className='text-xs text-muted-foreground'>
                    {user.reason}
                  </p>
                </div>
                <div className='text-right'>
                  <Badge variant={user.risk === 'high' ? 'destructive' : 'secondary'}>
                    {user.count}
                  </Badge>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
