/**
 * Le kit du style FerrisKey — les composants portés depuis `ferriskey-kit`.
 *
 * Il ne remplace pas `components/ui/` (shadcn) : il s'appuie dessus. Un
 * écran monte un `ListingPage` ou une `Section`, qui montent eux-mêmes des
 * `Button`, `Input`, `Checkbox` de shadcn.
 *
 * Trois composants du kit ne sont **pas** portés, parce que le produit a déjà
 * mieux — et qu'ajouter un doublon coûterait plus que la convergence qu'il
 * apporterait :
 *
 *  - `SaveBar`      → `components/ui/floating-action-bar.tsx`, déjà câblé aux
 *                     formulaires et animé (FK-06, étape 7 du plan de refonte).
 *  - `DangerZone`   → `components/danger-zone.tsx`, qui porte en plus la
 *                     confirmation bloquante que la version du kit n'avait pas.
 *  - `ProviderLogo` → `pages/identity-providers/components/provider-icon.tsx`,
 *                     dont le kit était précisément la copie.
 *  - `DurationInput`→ `components/ui/duration-input.tsx`.
 */
export { Pill, StatusDot, Squircle, Eyebrow, IconTile } from './primitives'
export type { PillTone } from './primitives'

export { Section } from './Section'
export { PageTabs } from './Tabs'
export type { TabItem } from './Tabs'
export { useRouteTabs } from './useRouteTabs'
export { Segmented } from './Segmented'
export type { SegmentedItem } from './Segmented'

export { ListingPage } from './ListingPage'
export type { ListingPageProps, ListingMetric, ListingAlert } from './ListingPage'
export { DataView } from './DataView'
export type { Column, CardSpec, ViewMode } from './DataView'
export { MetricsBand } from './MetricsBand'
export type { Metric } from './MetricsBand'
export { Sparkline } from './charts'
export type { ChartTone } from './charts'

export { EntityPicker } from './EntityPicker'
export type { PickableEntity } from './EntityPicker'
export { CreatePickerDialog } from './CreatePickerDialog'
export { useCreatePicker } from './useCreatePicker'

export {
  FieldRow,
  SwitchField,
  ChoiceCards,
  OrderedChoiceCards,
  ChipInput,
} from './form'
export type { Choice } from './form'
