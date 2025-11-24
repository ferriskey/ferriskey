import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Heading } from '@/components/ui/heading'
import {
  Shield,
  Clock,
  User,
  Search,
  Download,
  RefreshCw,
  Activity,
  Lock,
  Unlock,
  Globe,
  Smartphone,
  Monitor,
  Calendar,
} from 'lucide-react'
import { FlaggedUsers } from './flagged-users'
import { SecurityMetrics } from './security-metrics'
import { StrangeEventsAnalysis } from './strange-events-analysis'

export default function PageOverview() {
  return (
    <div className='flex flex-col gap-6 p-6 md:p-10 container mx-auto'>
      {/* Header */}
      <div className='flex items-center justify-between'>
        <div className='flex items-center gap-3'>
          <div className='h-8 w-8 rounded-full bg-primary/10 flex items-center justify-center'>
            <Shield className='size-5 text-primary' />
          </div>
          <div>
            <Heading size={3} weight='medium'>SeaWatch</Heading>
            <span className='text-sm text-muted-foreground'>
              Advanced security monitoring & audit system
            </span>
          </div>
        </div>

        <div className='flex items-center space-x-3'>
          <Button variant='outline' size='sm'>
            <Download className='h-4 w-4 mr-2' />
            Export
          </Button>
          <Button size='sm'>
            <RefreshCw className='h-4 w-4 mr-2' />
            Refresh
          </Button>
        </div>
      </div>

      {/* Security Metrics */}
      <SecurityMetrics />

      {/* Main Content */}
      <div className='grid grid-cols-1 lg:grid-cols-3 gap-6'>
        {/* Strange Events Analysis */}

        <StrangeEventsAnalysis />


        {/* Flagged Users */}
        <FlaggedUsers
          users={[
            {
              name: 'John Doe',
              email: 'john.doe@company.com',
              avatar: 'JD',
              reason: 'Multiple 403 errors',
              count: 15,
              risk: 'high'
            },
            {
              name: 'Sarah Wilson',
              email: 'sarah.wilson@company.com',
              avatar: 'SW',
              reason: 'Unusual access pattern',
              count: 8,
              risk: 'medium'
            },
            {
              name: 'Mike Johnson',
              email: 'mike.johnson@company.com',
              avatar: 'MJ',
              reason: 'Failed login attempts',
              count: 12,
              risk: 'high'
            }
          ]}
        />

      </div>

      {/* Audit Logs */}
      <Card>
        <CardHeader className='pb-4'>
          <div className='flex items-center justify-between'>
            <CardTitle className='flex items-center gap-2'>
              <Activity className='h-4 w-4 text-blue-500' />
              Recent Audit Logs
            </CardTitle>
            <div className='flex items-center space-x-3'>
              <div className='relative'>
                <Search className='h-4 w-4 absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground' />
                <Input
                  placeholder='Search events...'
                  className='pl-10 w-64'
                />
              </div>
              <Button variant='outline' size='sm'>
                <Calendar className='h-4 w-4 mr-2' />
                Last 24h
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className='space-y-3'>
            {[
              {
                action: 'Login Attempt',
                user: 'john.doe@company.com',
                result: 'Failed',
                ip: '192.168.1.100',
                device: 'Chrome on Windows',
                time: '2024-01-15 14:23:45',
                status: 'failed'
              },
              {
                action: 'Password Reset',
                user: 'sarah.wilson@company.com',
                result: 'Success',
                ip: '10.0.0.15',
                device: 'Safari on macOS',
                time: '2024-01-15 14:20:12',
                status: 'success'
              },
              {
                action: 'API Access',
                user: 'system@ferriskey.rs',
                result: 'Forbidden',
                ip: '203.45.67.89',
                device: 'API Client',
                time: '2024-01-15 14:18:33',
                status: 'failed'
              },
              {
                action: 'Role Assignment',
                user: 'admin@company.com',
                result: 'Success',
                ip: '192.168.1.50',
                device: 'Firefox on Ubuntu',
                time: '2024-01-15 14:15:08',
                status: 'success'
              },
              {
                action: 'Login Attempt',
                user: 'mike.johnson@company.com',
                result: 'Success',
                ip: '10.0.0.25',
                device: 'Mobile App',
                time: '2024-01-15 14:12:56',
                status: 'success'
              }
            ].map((log, index) => (
              <div key={index} className='border rounded-md p-3 py-6 hover:shadow-md transition-all'>
                <div className='flex items-center justify-between'>
                  <div className='flex items-center space-x-3'>
                    <div className={`h-8 w-8 rounded-full flex items-center justify-center ${log.status === 'success'
                      ? 'bg-green-500/10'
                      : 'bg-red-500/10'
                      }`}>
                      {log.status === 'success' ? (
                        <Unlock className='h-4 w-4 text-green-500' />
                      ) : (
                        <Lock className='h-4 w-4 text-red-500' />
                      )}
                    </div>
                    <div>
                      <div className='flex items-center space-x-2'>
                        <h4 className='font-medium'>
                          {log.action}
                        </h4>
                        <Badge variant={log.status === 'success' ? 'default' : 'destructive'}>
                          {log.result}
                        </Badge>
                      </div>
                      <div className='flex items-center space-x-4 text-sm text-muted-foreground mt-1'>
                        <span className='flex items-center gap-1'>
                          <User className='h-3 w-3' />
                          {log.user}
                        </span>
                        <span className='flex items-center gap-1'>
                          <Globe className='h-3 w-3' />
                          {log.ip}
                        </span>
                        <span className='flex items-center gap-1'>
                          {log.device.includes('Mobile') ? (
                            <Smartphone className='h-3 w-3' />
                          ) : (
                            <Monitor className='h-3 w-3' />
                          )}
                          {log.device}
                        </span>
                      </div>
                    </div>
                  </div>
                  <div className='text-right'>
                    <span className='text-sm text-muted-foreground flex items-center gap-1'>
                      <Clock className='h-3 w-3' />
                      {log.time}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
