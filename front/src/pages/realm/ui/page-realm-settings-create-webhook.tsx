import BlockContent from '@/components/ui/block-content'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Heading } from '@/components/ui/heading'
import { InputText } from '@/components/ui/input-text'
import { ScrollArea } from '@/components/ui/scroll-area'
import { TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Tabs } from '@radix-ui/react-tabs'
import { ArrowLeft } from 'lucide-react'
import { useFormContext } from 'react-hook-form'
import { CreateWebhookSchema } from '../validators'

export interface PageRealmSettingsCreateWebhookProps {}

export type WebhookEvent =
  | 'user.created'
  | 'user.updated'
  | 'user.deleted'
  | 'user.login'
  | 'user.logout'
  | 'role.created'
  | 'role.updated'
  | 'role.deleted'
  | 'client.created'
  | 'client.updated'
  | 'client.deleted'

const WEBHOOK_EVENTS: {
  category: string
  events: { key: WebhookEvent; label: string; description: string }[]
}[] = [
  {
    category: 'User Events',
    events: [
      {
        key: 'user.created',
        label: 'User Created',
        description: 'Triggered when a new user is registered',
      },
      {
        key: 'user.updated',
        label: 'User Updated',
        description: 'Triggered when user profile is modified',
      },
      {
        key: 'user.deleted',
        label: 'User Deleted',
        description: 'Triggered when a user account is deleted',
      },
      {
        key: 'user.login',
        label: 'User Login',
        description: 'Triggered when a user successfully logs in',
      },
      { key: 'user.logout', label: 'User Logout', description: 'Triggered when a user logs out' },
    ],
  },
  {
    category: 'Role Events',
    events: [
      {
        key: 'role.created',
        label: 'Role Created',
        description: 'Triggered when a new role is created',
      },
      {
        key: 'role.updated',
        label: 'Role Updated',
        description: 'Triggered when role permissions are modified',
      },
      {
        key: 'role.deleted',
        label: 'Role Deleted',
        description: 'Triggered when a role is deleted',
      },
    ],
  },
  {
    category: 'Client Events',
    events: [
      {
        key: 'client.created',
        label: 'Client Created',
        description: 'Triggered when a new client application is registered',
      },
      {
        key: 'client.updated',
        label: 'Client Updated',
        description: 'Triggered when client configuration is modified',
      },
      {
        key: 'client.deleted',
        label: 'Client Deleted',
        description: 'Triggered when a client application is deleted',
      },
    ],
  },
]

export default function PageRealmSettingsCreateWebhook({}: PageRealmSettingsCreateWebhookProps) {
  const form = useFormContext<CreateWebhookSchema>()

  return (
    <div className="flex flex-col p-4 gap-4">
      <div className="flex items-center gap-3">
        <Button
          variant="ghost"
          size="icon"
          //onClick={handleBack}
        >
          <ArrowLeft className="h-3 w-3" />
        </Button>
        <span className="text-gray-500 text-sm font-medium">Back to webhooks</span>
      </div>

      <div className="flex flex-col mb-4">
        <Heading size={3} className="text-gray-800">
          Create Webhook
        </Heading>

        <p className="text-sm text-gray-500 mt-1">
          Fill out the form below to create a new webhook.
        </p>
      </div>

      <div className="lg:w-1/3">
        <BlockContent title="General Details">
          <div className="flex flex-col gap-5">
            <InputText label="Webhook Name" name="name" />
            <InputText label="Webhook URL" name="url" />
          </div>
        </BlockContent>
      </div>

      <div>
        <BlockContent className="rounded-none" classNameContent="p-0" title="Events to subscribe">
          <Tabs defaultValue={WEBHOOK_EVENTS[0].category} className="flex">
            <TabsList asChild>
              <ScrollArea className="h-[400px] rounded-none w-[200px] bg-background border-r border-neutral-250 px-3 py-2">
                <p className="text-sm text-muted-foreground">Events</p>
                <div className="flex flex-col pt-3">
                  {WEBHOOK_EVENTS.map((event) => (
                    <TabsTrigger key={event.category} value={event.category} asChild>
                      <div className="justify-start py-1.5 px-2 data-[state=active]:bg-primary/10 data-[state=active]:text-primary !shadow-none rounded-sm">
                        {event.category}
                      </div>
                    </TabsTrigger>
                  ))}
                </div>
              </ScrollArea>
            </TabsList>
            <div className="flex-1 px-5 bg-background">
              {WEBHOOK_EVENTS.map((event) => (
                <TabsContent key={event.category} value={event.category}>
                  <ScrollArea className="h-[400px] rounded-none border-r border-neutral-250 py-2">
                    <p className="text-sm text-muted-foreground">Trigger sets</p>
                    <div className="flex flex-col gap-3 pt-3">
                      {event.events.map((event) => (
                        <div key={event.key} className="flex items-center gap-3">
                          <Checkbox id={event.key} />
                          <div>
                            <label htmlFor={event.key}>{event.label}</label>
                            <p className="text-xs text-muted-foreground">{event.description}</p>
                          </div>
                        </div>
                      ))}
                    </div>
                  </ScrollArea>
                </TabsContent>
              ))}
            </div>
          </Tabs>
        </BlockContent>
      </div>
    </div>
  )
}
