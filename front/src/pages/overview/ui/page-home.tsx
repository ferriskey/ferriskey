import { cn } from "@/lib/utils"
import { Clock, Columns3, Pyramid, ShieldUser, TablePropertiesIcon, Users } from "lucide-react"

export interface PageHomeProps {}

export default function PageHome({}: PageHomeProps) {
  return (
    <div className="flex flex-col gap-6 p-6 md:p-10">

      <div className="flex flex-col gap-6">
        <h1 className="text-3xl font-semibold">Welcome to FerrisKey</h1>

        <p className="text-sm text-gray-700 w-2/5">FerrisKey provides user federation, strong authentication, user management, fine-grained authorization, and more. Add authentication to applications and secure services with minimum effort. No need to deal with storing users or authenticating users.</p>
      </div>
      <StartingPoints />
    </div>
  )
}

const items = [
  {
    title: 'Create a Client',
    description: 'Register your application to enable secure authentication and authorization.',
    icon: Pyramid,
    background: 'bg-pink-500',
  },
  {
    title: 'Create a User',
    description: 'Add new users with customizable profiles and permission settings.',
    icon: Users,
    background: 'bg-yellow-500',
  },
  {
    title: 'Create a Role',
    description: 'Define access control roles to manage permissions across your applications.',
    icon: ShieldUser,
    background: 'bg-green-500',
  },
  {
    title: 'Manage realm settings',
    description: 'Configure authentication policies, session timeouts, and security options.',
    icon: Columns3,
    background: 'bg-blue-500',
  },
  {
    title: 'Manage Identity Providers',
    description: 'Set up social logins and external authentication services for your users.',
    icon: TablePropertiesIcon,
    background: 'bg-indigo-500',
  },
  {
    title: 'View audit logs',
    description: 'Monitor user activity and security events for compliance and troubleshooting.',
    icon: Clock,
    background: 'bg-purple-500',
  },
]

function StartingPoints() {
  return (
    <div>
        <ul role="list" className="mt-6 grid grid-cols-1 gap-6 border-b border-t border-gray-200 py-6 sm:grid-cols-2">
        {items.map((item, itemIdx) => (
          <li key={itemIdx} className="flow-root">
            <div className="relative -m-2 flex items-center space-x-4 rounded-xl p-2 focus-within:ring-2 focus-within:ring-indigo-500 hover:bg-gray-50">
              <div
                className={cn(item.background, 'flex size-16 shrink-0 items-center justify-center rounded-lg')}
              >
                <item.icon aria-hidden="true" className="size-6 text-white" />
              </div>
              <div>
                <h3 className="text-sm font-medium text-gray-900">
                  <a href="#" className="focus:outline-none">
                    <span aria-hidden="true" className="absolute inset-0" />
                    <span>{item.title}</span>
                    <span aria-hidden="true"> &rarr;</span>
                  </a>
                </h3>
                <p className="mt-1 text-sm text-gray-500">{item.description}</p>
              </div>
            </div>
          </li>
        ))}
      </ul>
      <div className="mt-4 flex">
        <a href="#" className="text-sm font-medium text-indigo-600 hover:text-indigo-500">
          Or start from an empty project
          <span aria-hidden="true"> &rarr;</span>
        </a>
      </div>
    </div>
  )
}
