import { Checkbox } from '@/components/ui/checkbox'
import { InputText } from '@/components/ui/input-text'
import { ScrollArea } from '@/components/ui/scroll-area'
import { TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Tabs } from '@radix-ui/react-tabs'
import { ArrowLeft } from 'lucide-react'
import { useFormContext } from 'react-hook-form'
import { CreateWebhookSchema } from '../validators'
import { Schemas } from '@/api/api.client'
import { WebhookCategory } from '@/utils/webhook-utils'
import WebhookTrigger = Schemas.WebhookTrigger
import { FormField } from '@/components/ui/form'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import ManageWebhookHeaders from '../components/manage-webhook-headers'

export interface PageRealmSettingsCreateWebhookProps {
  webhoobCategories: WebhookCategory[]
  handleTriggerToggle: (trigger: WebhookTrigger) => void
  isTriggerSelected: (trigger: WebhookTrigger) => boolean
  onSubmit: () => void
  handleBack: () => void
}

export default function PageRealmSettingsCreateWebhook({
  webhoobCategories,
  handleTriggerToggle,
  isTriggerSelected,
  onSubmit,
  handleBack,
}: PageRealmSettingsCreateWebhookProps) {
  const form = useFormContext<CreateWebhookSchema>()
  const formIsValid = form.formState.isValid

  return (
    <div className='flex flex-col gap-6 p-8'>
      {/* Header */}
      <div className='-mx-8 -mt-8 px-8 pt-8 pb-4 border-b'>
        <button
          onClick={handleBack}
          className='flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors mb-2'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Webhooks
        </button>
        <h1 className='text-2xl font-bold tracking-tight'>New Webhook</h1>
        <p className='text-sm text-muted-foreground mt-1'>
          Configure an endpoint to receive real-time event notifications.
        </p>
      </div>

      {/* General Details */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Webhook configuration</p>
          <h2 className='text-base font-semibold'>General Details</h2>
        </div>

        <FormField
          control={form.control}
          name='name'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Webhook Name</p>
                <p className='text-sm text-muted-foreground mt-0.5'>A descriptive name for this webhook.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Webhook Name' {...field} />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='endpoint'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Endpoint URL</p>
                <p className='text-sm text-muted-foreground mt-0.5'>The HTTPS URL that will receive events.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Webhook URL' {...field} />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='description'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Description</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Optional description for this webhook.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Description' {...field} />
              </div>
            </div>
          )}
        />
      </div>

      {/* HTTP Headers */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Request configuration</p>
          <h2 className='text-base font-semibold'>HTTP Headers</h2>
        </div>

        <div className='py-4 border-t'>
          <FormField
            control={form.control}
            name='headers'
            render={({ field }) => (
              <ManageWebhookHeaders
                headers={field.value || []}
                onChange={field.onChange}
              />
            )}
          />
        </div>
      </div>

      {/* Events */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Subscriptions</p>
          <h2 className='text-base font-semibold'>Events to subscribe</h2>
        </div>

        <div className='border rounded-md overflow-hidden'>
          <Tabs defaultValue={webhoobCategories[0].category} className='flex'>
            <TabsList asChild>
              <ScrollArea className='h-[400px] rounded-none w-[200px] bg-muted/30 border-r px-3 py-2'>
                <p className='text-xs text-muted-foreground mb-2'>Categories</p>
                <div className='flex flex-col gap-0.5'>
                  {webhoobCategories.map((value) => (
                    <TabsTrigger key={value.category} value={value.category} asChild>
                      <button className='w-full text-left text-sm py-1.5 px-2 data-[state=active]:bg-primary/10 data-[state=active]:text-primary rounded-sm transition-colors hover:bg-muted'>
                        {value.category}
                      </button>
                    </TabsTrigger>
                  ))}
                </div>
              </ScrollArea>
            </TabsList>
            <div className='flex-1 px-5 bg-background'>
              {webhoobCategories.map((value) => (
                <TabsContent key={value.category} value={value.category}>
                  <ScrollArea className='h-[400px] py-3'>
                    <p className='text-xs text-muted-foreground mb-3'>Trigger sets</p>
                    <div className='flex flex-col gap-3'>
                      {value.events.map((event) => (
                        <div key={event.key} className='flex items-start gap-3'>
                          <Checkbox
                            id={event.key}
                            checked={isTriggerSelected(event.key)}
                            onCheckedChange={() => handleTriggerToggle(event.key)}
                            className='mt-0.5'
                          />
                          <div>
                            <label htmlFor={event.key} className='text-sm font-medium cursor-pointer'>{event.label}</label>
                            <p className='text-xs text-muted-foreground mt-0.5'>{event.description}</p>
                          </div>
                        </div>
                      ))}
                    </div>
                  </ScrollArea>
                </TabsContent>
              ))}
            </div>
          </Tabs>
        </div>
      </div>

      <FloatingActionBar
        title='Create Webhook'
        show={formIsValid}
        actions={[{ label: 'Create', onClick: onSubmit }]}
        description='Create a webhook for your realm'
        onCancel={handleBack}
      />
    </div>
  )
}
