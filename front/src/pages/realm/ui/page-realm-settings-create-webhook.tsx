import BadgeColor from "@/components/ui/badge-color";
import { BadgeColorScheme } from "@/components/ui/badge-color.enum";
import BlockContent from "@/components/ui/block-content";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Heading } from "@/components/ui/heading";
import { InputText } from "@/components/ui/input-text";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { ArrowLeft } from "lucide-react";

export interface PageRealmSettingsCreateWebhookProps { }

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
  | 'client.deleted';

const WEBHOOK_EVENTS: { category: string; events: { key: WebhookEvent; label: string; description: string }[] }[] = [
  {
    category: 'User Events',
    events: [
      { key: 'user.created', label: 'User Created', description: 'Triggered when a new user is registered' },
      { key: 'user.updated', label: 'User Updated', description: 'Triggered when user profile is modified' },
      { key: 'user.deleted', label: 'User Deleted', description: 'Triggered when a user account is deleted' },
      { key: 'user.login', label: 'User Login', description: 'Triggered when a user successfully logs in' },
      { key: 'user.logout', label: 'User Logout', description: 'Triggered when a user logs out' }
    ]
  },
  {
    category: 'Role Events',
    events: [
      { key: 'role.created', label: 'Role Created', description: 'Triggered when a new role is created' },
      { key: 'role.updated', label: 'Role Updated', description: 'Triggered when role permissions are modified' },
      { key: 'role.deleted', label: 'Role Deleted', description: 'Triggered when a role is deleted' }
    ]
  },
  {
    category: 'Client Events',
    events: [
      { key: 'client.created', label: 'Client Created', description: 'Triggered when a new client application is registered' },
      { key: 'client.updated', label: 'Client Updated', description: 'Triggered when client configuration is modified' },
      { key: 'client.deleted', label: 'Client Deleted', description: 'Triggered when a client application is deleted' }
    ]
  }
];

export default function PageRealmSettingsCreateWebhook({ }: PageRealmSettingsCreateWebhookProps) {
  return (
    <div className='flex flex-col p-4 gap-4'>
      <div className='flex items-center gap-3'>
        <Button
          variant='ghost'
          size='icon'
        //onClick={handleBack}
        >
          <ArrowLeft className='h-3 w-3' />
        </Button>
        <span className='text-gray-500 text-sm font-medium'>Back to webhooks</span>
      </div>

      <div className='flex flex-col mb-4'>
        <Heading size={3} className='text-gray-800'>
          Create Webhook
        </Heading>

        <p className='text-sm text-gray-500 mt-1'>
          Fill out the form below to create a new webhook.
        </p>
        {/* Add your form here */}
      </div>

      <div className='lg:w-1/3'>

        <BlockContent title='General Details'>
          <div className='flex flex-col gap-5'>

            <InputText
              label='Webhook Name'
              name='name'
            />

            <InputText
              label='Webhook URL'
              name='url'
            />
          </div>
        </BlockContent>




      </div>

      <div>
        <BlockContent title='Events to subscribe'>
          <div>
            <ScrollArea className='h-[500px] rounded-md border bg-background'>
              <div className='p-4 space-y-4'>
                {WEBHOOK_EVENTS.map((category, index) => (
                  <div key={category.category} className='space-y-3'>

                    <div className='flex items-center justify-between'>
                      <div className='flex items-center space-x-2'>
                        <Checkbox
                          id={`event-${category.category}`}
                        />


                        <Label
                          className='text-sm font-medium cursor-pointer'
                        >
                          {category.category}
                        </Label>
                      </div>

                      <BadgeColor color={BadgeColorScheme.GRAY} className='text-xs'>
                        0/{category.events.length}
                      </BadgeColor>
                    </div>

                    <div className='ml-6 space-y-2'>
                      {category.events.map((event, index) => (
                        <div key={index} className='flex items-center space-x-2'>
                          <Checkbox
                            id={`event-${category.category}`}
                          />

                          <Label
                            className='text-sm cursor-pointer flex-1'
                          >
                            {event.label}
                          </Label>
                        </div>
                      ))}
                    </div>

                    {index != WEBHOOK_EVENTS.length - 1 && (
                      <Separator />
                    )}
                  </div>
                ))}
              </div>
            </ScrollArea>

          </div>

        </BlockContent>
      </div>
    </div >
  );
}
