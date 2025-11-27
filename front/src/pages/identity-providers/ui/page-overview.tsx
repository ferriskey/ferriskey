import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Heading } from '@/components/ui/heading'

import {
  ScanFace,
  Search,
  Plus,
  Download,
  RefreshCw,
  Settings,
  Shield,
  Globe,
  Key,
  Users,
  Activity,
  CheckCircle,

  Clock,
} from 'lucide-react'
import { ComingSoon } from '@/components/ui/coming-soon'

export default function PageOverview() {
  return (
    <ComingSoon
      title='Identity Providers Management'
      description='Comprehensive identity provider management with support for OIDC, SAML, LDAP, and social providers.'
      badgeText='Coming Soon'
      bulletPoints={[
        { text: 'Multiple protocols:', highlight: 'OIDC, SAML, LDAP, and social authentication' },
        { text: 'Seamless integration:', highlight: 'connect with Google, Microsoft, GitHub and more' },
        { text: 'Advanced mapping:', highlight: 'flexible user attribute and role mapping' },
        { text: 'Single Sign-On:', highlight: 'unified authentication across applications' },
        { text: 'Enterprise ready:', highlight: 'supports Active Directory and enterprise LDAP' },
        { text: 'Security first:', highlight: 'token validation and secure credential handling' }
      ]}
      imageSrc='/logo_ferriskey.png'
      imageAlt='Identity Providers management interface'
      blurIntensity='medium'
    >
      <div className='flex flex-col gap-6 p-6 md:p-10 container mx-auto'>
        {/* Header */}
        <div className='flex items-center justify-between'>
          <div className='flex items-center gap-3'>
            <div className='h-8 w-8 rounded-full bg-primary/10 flex items-center justify-center'>
              <ScanFace className='size-5 text-primary' />
            </div>
            <div>
              <Heading size={3} weight='medium'>Identity Providers</Heading>
              <span className='text-sm text-muted-foreground'>
                Manage external authentication sources and SSO integrations
              </span>
            </div>
          </div>

          <div className='flex items-center space-x-3'>
            <Button variant='outline' size='sm'>
              <Download className='h-4 w-4 mr-2' />
              Export Config
            </Button>
            <Button size='sm'>
              <Plus className='h-4 w-4 mr-2' />
              Add Provider
            </Button>
          </div>
        </div>

        {/* Provider Stats */}
        <div className='grid grid-cols-1 md:grid-cols-4 gap-4'>
          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
              <CardTitle className='text-sm font-medium'>Total Providers</CardTitle>
              <ScanFace className='h-4 w-4 text-muted-foreground' />
            </CardHeader>
            <CardContent>
              <div className='text-2xl font-bold'>8</div>
              <p className='text-xs text-muted-foreground'>3 active, 2 testing</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
              <CardTitle className='text-sm font-medium'>Active Connections</CardTitle>
              <Activity className='h-4 w-4 text-muted-foreground' />
            </CardHeader>
            <CardContent>
              <div className='text-2xl font-bold'>1,234</div>
              <p className='text-xs text-muted-foreground'>+12% from last week</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
              <CardTitle className='text-sm font-medium'>SSO Sessions</CardTitle>
              <Key className='h-4 w-4 text-muted-foreground' />
            </CardHeader>
            <CardContent>
              <div className='text-2xl font-bold'>456</div>
              <p className='text-xs text-muted-foreground'>Active sessions today</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
              <CardTitle className='text-sm font-medium'>Success Rate</CardTitle>
              <CheckCircle className='h-4 w-4 text-muted-foreground' />
            </CardHeader>
            <CardContent>
              <div className='text-2xl font-bold'>99.2%</div>
              <p className='text-xs text-muted-foreground'>Last 30 days</p>
            </CardContent>
          </Card>
        </div>

        {/* Main Content */}
        <div className='grid grid-cols-1 lg:grid-cols-3 gap-6'>
          {/* Active Providers */}
          <div className='lg:col-span-2'>
            <Card>
              <CardHeader className='pb-4'>
                <div className='flex items-center justify-between'>
                  <CardTitle className='flex items-center gap-2'>
                    <ScanFace className='h-4 w-4 text-blue-500' />
                    Active Identity Providers
                  </CardTitle>
                  <div className='flex items-center space-x-3'>
                    <div className='relative'>
                      <Search className='h-4 w-4 absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground' />
                      <Input
                        placeholder='Search providers...'
                        className='pl-10 w-64'
                      />
                    </div>
                    <Button variant='outline' size='sm'>
                      <Settings className='h-4 w-4 mr-2' />
                      Manage
                    </Button>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className='space-y-4'>
                  {[
                    {
                      name: 'Google Workspace',
                      type: 'OIDC',
                      status: 'active',
                      users: 145,
                      lastSync: '2 minutes ago',
                      icon: '🔍',
                      domain: 'company.com'
                    },
                    {
                      name: 'Microsoft Azure AD',
                      type: 'SAML',
                      status: 'active',
                      users: 89,
                      lastSync: '5 minutes ago',
                      icon: '🔷',
                      domain: 'company.onmicrosoft.com'
                    },
                    {
                      name: 'GitHub Enterprise',
                      type: 'OIDC',
                      status: 'testing',
                      users: 23,
                      lastSync: '1 hour ago',
                      icon: '🐙',
                      domain: 'github.company.com'
                    },
                    {
                      name: 'Corporate LDAP',
                      type: 'LDAP',
                      status: 'active',
                      users: 456,
                      lastSync: '10 minutes ago',
                      icon: '🏢',
                      domain: 'ldap.company.local'
                    },
                    {
                      name: 'Okta',
                      type: 'SAML',
                      status: 'warning',
                      users: 78,
                      lastSync: '2 hours ago',
                      icon: '🔐',
                      domain: 'company.okta.com'
                    }
                  ].map((provider, index) => (
                    <div key={index} className='border rounded-lg p-4 hover:shadow-md transition-all'>
                      <div className='flex items-center justify-between'>
                        <div className='flex items-center space-x-4'>
                          <div className='text-2xl'>{provider.icon}</div>
                          <div>
                            <div className='flex items-center space-x-2'>
                              <h4 className='font-medium'>{provider.name}</h4>
                              <Badge variant={provider.type === 'OIDC' ? 'default' : provider.type === 'SAML' ? 'secondary' : 'outline'}>
                                {provider.type}
                              </Badge>
                              <Badge
                                variant={
                                  provider.status === 'active' ? 'default' :
                                    provider.status === 'testing' ? 'secondary' :
                                      'destructive'
                                }
                              >
                                {provider.status}
                              </Badge>
                            </div>
                            <div className='flex items-center space-x-4 text-sm text-muted-foreground mt-1'>
                              <span className='flex items-center gap-1'>
                                <Globe className='h-3 w-3' />
                                {provider.domain}
                              </span>
                              <span className='flex items-center gap-1'>
                                <Users className='h-3 w-3' />
                                {provider.users} users
                              </span>
                              <span className='flex items-center gap-1'>
                                <Clock className='h-3 w-3' />
                                {provider.lastSync}
                              </span>
                            </div>
                          </div>
                        </div>
                        <div className='flex items-center space-x-2'>
                          <Button variant='outline' size='sm'>
                            <Settings className='h-4 w-4' />
                          </Button>
                          <Button variant='outline' size='sm'>
                            <RefreshCw className='h-4 w-4' />
                          </Button>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Provider Types & Quick Actions */}
          <div className='space-y-6'>
            {/* Provider Types */}
            <Card>
              <CardHeader>
                <CardTitle className='text-lg'>Provider Types</CardTitle>
              </CardHeader>
              <CardContent>
                <div className='space-y-4'>
                  {[
                    { type: 'OIDC', count: 3, color: 'bg-blue-500', description: 'OpenID Connect' },
                    { type: 'SAML', count: 2, color: 'bg-green-500', description: 'SAML 2.0' },
                    { type: 'LDAP', count: 1, color: 'bg-purple-500', description: 'LDAP/AD' },
                    { type: 'Social', count: 2, color: 'bg-orange-500', description: 'Social Login' }
                  ].map((item, index) => (
                    <div key={index} className='flex items-center justify-between'>
                      <div className='flex items-center space-x-3'>
                        <div className={`w-3 h-3 rounded-full ${item.color}`} />
                        <div>
                          <span className='font-medium'>{item.type}</span>
                          <p className='text-xs text-muted-foreground'>{item.description}</p>
                        </div>
                      </div>
                      <Badge variant='outline'>{item.count}</Badge>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* Recent Activity */}
            <Card>
              <CardHeader>
                <CardTitle className='text-lg'>Recent Activity</CardTitle>
              </CardHeader>
              <CardContent>
                <div className='space-y-3'>
                  {[
                    { action: 'Provider sync', provider: 'Google Workspace', status: 'success', time: '2 min ago' },
                    { action: 'User login', provider: 'Azure AD', status: 'success', time: '5 min ago' },
                    { action: 'Configuration update', provider: 'GitHub', status: 'warning', time: '1 hour ago' },
                    { action: 'Token refresh', provider: 'Okta', status: 'failed', time: '2 hours ago' }
                  ].map((activity, index) => (
                    <div key={index} className='flex items-center space-x-3 p-2 rounded-lg bg-muted/50'>
                      <div className={`w-2 h-2 rounded-full ${activity.status === 'success' ? 'bg-green-500' :
                        activity.status === 'warning' ? 'bg-yellow-500' : 'bg-red-500'
                        }`} />
                      <div className='flex-1'>
                        <p className='text-sm font-medium'>{activity.action}</p>
                        <p className='text-xs text-muted-foreground'>{activity.provider}</p>
                      </div>
                      <span className='text-xs text-muted-foreground'>{activity.time}</span>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* Quick Actions */}
            <Card>
              <CardHeader>
                <CardTitle className='text-lg'>Quick Actions</CardTitle>
              </CardHeader>
              <CardContent className='space-y-2'>
                <Button className='w-full justify-start' variant='outline'>
                  <Plus className='h-4 w-4 mr-2' />
                  Add OIDC Provider
                </Button>
                <Button className='w-full justify-start' variant='outline'>
                  <Shield className='h-4 w-4 mr-2' />
                  Configure SAML
                </Button>
                <Button className='w-full justify-start' variant='outline'>
                  <Globe className='h-4 w-4 mr-2' />
                  Test Connection
                </Button>
                <Button className='w-full justify-start' variant='outline'>
                  <Download className='h-4 w-4 mr-2' />
                  Export Metadata
                </Button>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </ComingSoon>
  )
}
