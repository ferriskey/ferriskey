import BlockContent from '@/components/ui/block-content'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Heading } from '@/components/ui/heading'
import { InputText } from '@/components/ui/input-text'
import { ScrollArea } from '@/components/ui/scroll-area'
import { TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Tabs } from '@radix-ui/react-tabs'
import { ArrowLeft, PlusIcon } from 'lucide-react'
import { useFormContext } from 'react-hook-form'
import { CreateWebhookSchema } from '../validators'
import { Schemas } from '@/api/api.client'
import { WebhookCategory } from '@/utils/webhook-utils'
import WebhookTrigger = Schemas.WebhookTrigger
import { FormField } from '@/components/ui/form'
import FloatingActionBar from '@/components/ui/floating-action-bar'

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
  handleBack
}: PageRealmSettingsCreateWebhookProps) {
  const form = useFormContext<CreateWebhookSchema>()
  const formIsValid = form.formState.isValid

  console.log(formIsValid, form.getValues(), form.formState)


  return (
    <div className='flex flex-col p-4 gap-4'>
      <div className='flex items-center gap-3'>
        <Button
          variant='ghost'
          size='icon'
          onClick={handleBack}
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
      </div>

      <div className='lg:w-1/3'>
        <BlockContent title='General Details'>
          <div className='flex flex-col gap-5'>
            <FormField
              control={form.control}
              name='name'
              render={({ field }) => (
                <InputText label='Webhook Name' {...field} />
              )}
            />

            <FormField
              control={form.control}
              name='endpoint'
              render={({ field }) => (
                <InputText label='Webhook URL' {...field} />
              )}
            />

            <FormField
              control={form.control}
              name='description'
              render={({ field }) => (
                <InputText label='Webhook Description' {...field} />
              )}
            />
          </div>
        </BlockContent>
      </div>

      <div>
        <BlockContent className='rounded-none' classNameContent='p-0' title='Events to subscribe'>
          <Tabs defaultValue={webhoobCategories[0].category} className='flex'>
            <TabsList asChild>
              <ScrollArea className='h-[400px] rounded-none w-[200px] bg-background border-r border-neutral-250 px-3 py-2'>
                <p className='text-sm text-muted-foreground'>Events</p>
                <div className='flex flex-col pt-3'>
                  {webhoobCategories.map((value) => (
                    <TabsTrigger key={value.category} value={value.category} asChild>
                      <div className='justify-start py-1.5 px-2 data-[state=active]:bg-primary/10 data-[state=active]:text-primary !shadow-none rounded-sm'>
                        {value.category}
                      </div>
                    </TabsTrigger>
                  ))}
                </div>
              </ScrollArea>
            </TabsList>
            <div className='flex-1 px-5 bg-background'>
              {webhoobCategories.map((value) => (
                <TabsContent key={value.category} value={value.category}>
                  <ScrollArea className='h-[400px] rounded-none border-r border-neutral-250 py-2'>
                    <p className='text-sm text-muted-foreground'>Trigger sets</p>
                    <div className='flex flex-col gap-3 pt-3'>
                      {value.events.map((event) => (
                        <div key={event.key} className='flex items-center gap-3'>
                          <Checkbox id={event.key} checked={isTriggerSelected(event.key)} onCheckedChange={() => {
                            console.log(event.key)
                            handleTriggerToggle(event.key)
                          }} />
                          <div>
                            <label htmlFor={event.key}>{event.label}</label>
                            <p className='text-xs text-muted-foreground'>{event.description}</p>
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

        <FloatingActionBar
          title='Create Webhook'
          show={formIsValid}
          actions={[
            {
              label: 'Create',
              onClick: () => {
                onSubmit()
              }
            }
          ]}
          description='Create a webhook for your realm'
          icon={<PlusIcon className='w-4 h-4' />}
        />
      </div>
    </div>
  )
}
