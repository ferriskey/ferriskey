import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Activity, AlertTriangle, ChevronRight, Clock, Eye, Filter, MapPin, User } from 'lucide-react'

export function StrangeEventsAnalysis() {
  return (
    <div className='lg:col-span-2'>
      <Card>
        <CardHeader className='pb-4 blur-[4px]'>
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
        <CardContent className='blur-[4px]'>
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
  )
}
