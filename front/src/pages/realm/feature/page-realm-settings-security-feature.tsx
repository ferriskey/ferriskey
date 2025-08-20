import { useFeature } from '@/hooks/use-feature'
import { Feature } from '@/lib/features'

export default function PageRealmSettingsSecurityFeature() {
  const enabled = useFeature(Feature.REALM_SETTINGS)

  if (!enabled) return null
  return <div>Security</div>
}
