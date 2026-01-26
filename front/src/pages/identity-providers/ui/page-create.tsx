import { UseFormReturn } from 'react-hook-form'
import { ArrowLeft, ArrowRight, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import type { ProviderTemplate } from '@/constants/identity-provider-templates'
import ProviderWizardProgress, { WIZARD_STEPS } from '../components/provider-wizard-progress'
import ProviderGallery from '../components/provider-gallery'
import ProviderConfigForm, { type ProviderFormData } from '../components/provider-config-form'
import ProviderReview from '../components/provider-review'
import ProviderHelpPanel from '../components/provider-help-panel'

interface PageCreateProps {
  form: UseFormReturn<ProviderFormData>
  currentStep: number
  selectedTemplate: ProviderTemplate | null
  callbackUrl: string
  isPending: boolean
  onSelectTemplate: (template: ProviderTemplate) => void
  onNextStep: () => void
  onPrevStep: () => void
  onSubmit: () => void
  onCancel: () => void
}

export default function PageCreate({
  form,
  currentStep,
  selectedTemplate,
  callbackUrl,
  isPending,
  onSelectTemplate,
  onNextStep,
  onPrevStep,
  onSubmit,
  onCancel,
}: PageCreateProps) {
  const canProceedFromStep1 = selectedTemplate !== null
  const canProceedFromStep2 = form.formState.isValid

  return (
    <div className='flex flex-col p-4 md:p-8 gap-4 max-w-7xl mx-auto'>
      {/* Header */}
      <div className='flex items-center gap-4'>
        <Button variant='ghost' size='icon' onClick={onCancel}>
          <ArrowLeft className='h-4 w-4' />
        </Button>
        <div>
          <h1 className='text-xl font-semibold'>Add Identity Provider</h1>
          <p className='text-sm text-muted-foreground'>
            Connect an external authentication provider
          </p>
        </div>
      </div>

      {/* Progress Indicator */}
      <Card>
        <CardContent className='p-4'>
          <ProviderWizardProgress steps={WIZARD_STEPS} currentStep={currentStep} />
        </CardContent>
      </Card>

      {/* Step Content */}
      {currentStep === 1 && (
        <Card>
          <CardContent className='p-6'>
            <div className='space-y-4'>
              <div>
                <h2 className='text-base font-medium'>Select a Provider</h2>
                <p className='text-sm text-muted-foreground'>
                  Choose an identity provider to configure. We'll pre-fill the technical details for you.
                </p>
              </div>
              <ProviderGallery
                onSelect={onSelectTemplate}
                selectedId={selectedTemplate?.id}
              />
            </div>
          </CardContent>
        </Card>
      )}

      {currentStep === 2 && selectedTemplate && (
        <div className='grid grid-cols-1 lg:grid-cols-[1fr_400px] gap-4'>
          {/* Form Column */}
          <Card>
            <CardContent className='p-6'>
              <div className='space-y-4'>
                <div>
                  <h2 className='text-base font-medium'>Configure {selectedTemplate.displayName}</h2>
                  <p className='text-sm text-muted-foreground'>
                    Enter your OAuth credentials from the {selectedTemplate.displayName} developer console.
                  </p>
                </div>
                <ProviderConfigForm
                  template={selectedTemplate}
                  form={form}
                  callbackUrl={callbackUrl}
                />
              </div>
            </CardContent>
          </Card>

          {/* Help Panel Column */}
          <ProviderHelpPanel
            template={selectedTemplate}
            callbackUrl={callbackUrl}
          />
        </div>
      )}

      {currentStep === 3 && selectedTemplate && (
        <Card>
          <CardContent className='p-6'>
            <div className='space-y-4'>
              <div>
                <h2 className='text-base font-medium'>Review Configuration</h2>
                <p className='text-sm text-muted-foreground'>
                  Please review your settings before creating the provider.
                </p>
              </div>
              <ProviderReview
                template={selectedTemplate}
                formData={form.getValues()}
                callbackUrl={callbackUrl}
              />
            </div>
          </CardContent>
        </Card>
      )}

      {/* Navigation */}
      <div className='flex items-center justify-between'>
        <Button
          variant='outline'
          onClick={currentStep === 1 ? onCancel : onPrevStep}
        >
          <ArrowLeft className='h-4 w-4 mr-2' />
          {currentStep === 1 ? 'Cancel' : 'Back'}
        </Button>

        <div className='flex items-center gap-2'>
          {currentStep < 3 ? (
            <Button
              onClick={onNextStep}
              disabled={
                (currentStep === 1 && !canProceedFromStep1) ||
                (currentStep === 2 && !canProceedFromStep2)
              }
            >
              Next
              <ArrowRight className='h-4 w-4 ml-2' />
            </Button>
          ) : (
            <Button onClick={onSubmit} disabled={isPending}>
              {isPending && <Loader2 className='h-4 w-4 mr-2 animate-spin' />}
              Create Provider
            </Button>
          )}
        </div>
      </div>
    </div>
  )
}
