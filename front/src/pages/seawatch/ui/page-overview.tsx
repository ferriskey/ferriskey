import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Heading } from '@/components/ui/heading'
import {
  Shield,
  AlertTriangle,
  Eye,
  Clock,
  User,
  MapPin,
  Search,
  Filter,
  Download,
  RefreshCw,
  TrendingUp,
  TrendingDown,
  Activity,
  Lock,
  Unlock,
  Globe,
  Smartphone,
  Monitor,
  Calendar,
  ChevronRight
} from 'lucide-react'

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
      <div className='grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6'>
        <Card>
          <CardContent className='p-6'>
            <div className='flex items-center justify-between'>
              <div>
                <p className='text-sm font-medium text-muted-foreground'>
                  Critical Events
                </p>
                <p className='text-3xl font-bold text-red-600'>
                  23
                </p>
                <p className='text-xs text-red-500 flex items-center mt-1'>
                  <TrendingUp className='h-3 w-3 mr-1' />
                  +12% from yesterday
                </p>
              </div>
              <div className='h-8 w-8 rounded-full bg-red-500/10 flex items-center justify-center'>
                <AlertTriangle className='h-4 w-4 text-red-500' />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className='p-6'>
            <div className='flex items-center justify-between'>
              <div>
                <p className='text-sm font-medium text-muted-foreground'>
                  Failed Attempts
                </p>
                <p className='text-3xl font-bold text-amber-600'>
                  156
                </p>
                <p className='text-xs text-amber-500 flex items-center mt-1'>
                  <TrendingDown className='h-3 w-3 mr-1' />
                  -5% from yesterday
                </p>
              </div>
              <div className='h-8 w-8 rounded-full bg-amber-500/10 flex items-center justify-center'>
                <Lock className='h-4 w-4 text-amber-500' />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className='p-6'>
            <div className='flex items-center justify-between'>
              <div>
                <p className='text-sm font-medium text-muted-foreground'>
                  Active Sessions
                </p>
                <p className='text-3xl font-bold text-blue-600'>
                  1,247
                </p>
                <p className='text-xs text-blue-500 flex items-center mt-1'>
                  <Activity className='h-3 w-3 mr-1' />
                  Real-time monitoring
                </p>
              </div>
              <div className='h-8 w-8 rounded-full bg-blue-500/10 flex items-center justify-center'>
                <Eye className='h-4 w-4 text-blue-500' />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className='p-6'>
            <div className='flex items-center justify-between'>
              <div>
                <p className='text-sm font-medium text-muted-foreground'>
                  Blocked Threats
                </p>
                <p className='text-3xl font-bold text-green-600'>
                  89
                </p>
                <p className='text-xs text-green-500 flex items-center mt-1'>
                  <Shield className='h-3 w-3 mr-1' />
                  Last 24 hours
                </p>
              </div>
              <div className='h-8 w-8 rounded-full bg-green-500/10 flex items-center justify-center'>
                <Unlock className='h-4 w-4 text-green-500' />
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Main Content */}
      <div className='grid grid-cols-1 lg:grid-cols-3 gap-6'>
        {/* Strange Events Analysis */}
        <div className='lg:col-span-2'>
          <Card>
            <CardHeader className='pb-4'>
              <div className='flex items-center justify-between'>
                <CardTitle className='flex items-center gap-2'>
                  <AlertTriangle className='h-4 w-4 text-red-500' />
                  Strange Events Analysis
                </CardTitle>
                <Button variant='outline' size='sm'>
                  <Filter className='h-4 w-4 mr-2' />
                  Filters
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <Tabs defaultValue='critical' className='w-full'>
                <TabsList className='grid w-full grid-cols-3'>
                  <TabsTrigger value='critical'>Critical</TabsTrigger>
                  <TabsTrigger value='suspicious'>Suspicious</TabsTrigger>
                  <TabsTrigger value='anomalies'>Anomalies</TabsTrigger>
                </TabsList>

                <TabsContent value='critical' className='mt-6'>
                  <div className='space-y-3'>
                    {[
                      {
                        id: 1,
                        type: 'Multiple 403 Errors',
                        user: 'john.doe@company.com',
                        count: 15,
                        time: '2 mins ago',
                        severity: 'high',
                        location: 'US, New York'
                      },
                      {
                        id: 2,
                        type: 'Unusual Login Pattern',
                        user: 'admin@system.local',
                        count: 8,
                        time: '5 mins ago',
                        severity: 'critical',
                        location: 'RU, Moscow'
                      },
                      {
                        id: 3,
                        type: 'Brute Force Attempt',
                        user: 'unknown',
                        count: 45,
                        time: '12 mins ago',
                        severity: 'critical',
                        location: 'CN, Beijing'
                      }
                    ].map((event) => (
                      <div key={event.id} className='border rounded-md p-3 py-6 hover:shadow-md hover:cursor-pointer shadow-primary/10 transition-all'>
                        <div className='flex items-center justify-between'>
                          <div className='flex items-center space-x-3'>
                            <div className={`h-8 w-8 rounded-full flex items-center justify-center ${event.severity === 'critical'
                              ? 'bg-red-500/10'
                              : 'bg-amber-500/10'
                              }`}>
                              <AlertTriangle className={`h-4 w-4 ${event.severity === 'critical'
                                ? 'text-red-500'
                                : 'text-amber-500'
                                }`} />
                            </div>
                            <div>
                              <h4 className='font-medium'>
                                {event.type}
                              </h4>
                              <div className='flex items-center space-x-4 text-sm text-muted-foreground mt-1'>
                                <span className='flex items-center gap-1'>
                                  <User className='h-3 w-3' />
                                  {event.user}
                                </span>
                                <span className='flex items-center gap-1'>
                                  <MapPin className='h-3 w-3' />
                                  {event.location}
                                </span>
                              </div>
                            </div>
                          </div>
                          <div className='flex items-center space-x-3'>
                            <Badge variant={event.severity === 'critical' ? 'destructive' : 'default'}>
                              {event.count} events
                            </Badge>
                            <span className='text-sm text-muted-foreground flex items-center gap-1'>
                              <Clock className='h-3 w-3' />
                              {event.time}
                            </span>
                            <ChevronRight className='h-4 w-4 text-muted-foreground' />
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </TabsContent>

                <TabsContent value='suspicious' className='mt-6'>
                  <div className='text-center py-12 text-muted-foreground'>
                    <Eye className='h-12 w-12 mx-auto mb-4 opacity-50' />
                    <p>No suspicious activities detected</p>
                  </div>
                </TabsContent>

                <TabsContent value='anomalies' className='mt-6'>
                  <div className='text-center py-12 text-muted-foreground'>
                    <Activity className='h-12 w-12 mx-auto mb-4 opacity-50' />
                    <p>Analyzing patterns...</p>
                  </div>
                </TabsContent>
              </Tabs>
            </CardContent>
          </Card>
        </div>

        {/* Flagged Users */}
        <div>
          <Card>
            <CardHeader className='pb-4'>
              <CardTitle className='flex items-center gap-2'>
                <User className='h-4 w-4 text-amber-500' />
                Flagged Users
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='space-y-3'>
                {[
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
                ].map((user, index) => (
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
  );
}
