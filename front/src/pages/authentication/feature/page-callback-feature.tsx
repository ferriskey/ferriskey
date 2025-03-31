import { useEffect, useState } from 'react'
import PageCallback from '../ui/page-callback'

export default function PageCallbackFeature() {
  const [code, setCode] = useState<string | null>(null)
  const [setup, setSetup] = useState<boolean>(false)

  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search)
    const code = urlParams.get('code')
    setCode(code)

    setSetup(true)
  }, [])

  return <PageCallback code={code} setup={setup} />
}
