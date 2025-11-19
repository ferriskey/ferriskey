import { Card, CardContent } from "@/components/ui/card";
import { Activity, AlertTriangle, Eye, Shield, TrendingDown, TrendingUp, Unlock, Lock } from "lucide-react";

export function SecurityMetrics() {
  return (
    <div className='grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6'>
      <Card>
        <CardContent className='p-6'>
          <div className='flex items-center justify-between blur-[4px]'>
            <div>
              <p className='text-sm font-medium text-muted-foreground'>
                Critical Events
              </p>
              <p className='text-3xl font-bold text-red-600'>
                45
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
          <div className='flex items-center justify-between blur-[4px]'>
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
          <div className='flex items-center justify-between blur-[4px]'>
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
          <div className='flex items-center justify-between blur-[4px]'>
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
  )
}
