export {
  formatDateTime,
  formatTime,
  formatRelative,
  formatTimestamp,
} from '@/utils/format-date'

import { translate } from '@/lib/i18n'
import { Schemas } from '@/api/api.client'

import CompassFlow = Schemas.CompassFlow
import CompassFlowStep = Schemas.CompassFlowStep
import FlowStepName = Schemas.FlowStepName
import FlowStatus = Schemas.FlowStatus
import StepStatus = Schemas.StepStatus
import type { PillTone } from '@/components/kit'

export const catalogedStepNames: Record<FlowStepName, true> = {
  authorize: true,
  credential_validation: true,
  mfa_challenge: true,
  token_exchange: true,
  idp_redirect: true,
  idp_callback: true,
  finalize: true,
  saml_authn_request: true,
  saml_assertion: true,
}

export const flowStatusTone: Record<FlowStatus, PillTone> = {
  success: 'success',
  failure: 'danger',
  expired: 'amber',
  pending: 'info',
}

export const stepStatusTone: Record<StepStatus, PillTone> = {
  success: 'success',
  failure: 'danger',
  skipped: 'neutral',
}

export const stepLabel = (step: CompassFlowStep) =>
  catalogedStepNames[step.step_name]
    ? translate(`compass:step.${step.step_name}.label`)
    : step.step_name

export const stepDescription = (step: CompassFlowStep) =>
  catalogedStepNames[step.step_name]
    ? translate(`compass:step.${step.step_name}.description`)
    : null

export const formatDuration = (ms?: number | null) => {
  if (ms == null) return '—'
  return ms < 1000
    ? translate('compass:duration.milliseconds', { value: ms })
    : translate('compass:duration.seconds', { value: (ms / 1000).toFixed(1) })
}

export const orderedSteps = (flow: CompassFlow) =>
  [...flow.steps].sort(
    (a, b) => new Date(a.started_at).getTime() - new Date(b.started_at).getTime()
  )

export const failingStep = (flow: CompassFlow): CompassFlowStep | null =>
  orderedSteps(flow).find((step) => step.status === 'failure') ?? null
