import useUiModeStore from '@/store/ui-mode.store'
import Layout from './layout'
import ProductLayout from './product-layout'

export default function LayoutSwitch() {
  const mode = useUiModeStore((s) => s.mode)
  return mode === 'product' ? <ProductLayout /> : <Layout />
}
