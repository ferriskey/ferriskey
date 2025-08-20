import { Feature } from '@/lib/features'

export function useFeature(feature: Feature) {
  // We might want this to be dynamic in the future by making able the user to enable/disable features with a back-office
  return import.meta.env.VITE_FEATURES?.includes(feature) ?? false
}
