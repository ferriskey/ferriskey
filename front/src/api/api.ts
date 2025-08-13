import { createApiClient } from '@/api/api.zod.ts'
import { TanstackQueryApiClient } from '@/api/api.tanstack.ts'

const api = createApiClient((method, url, params) =>
  fetch(url, { method, body: JSON.stringify(params) }).then((res) => res.json())
)

export const tanstackApi = new TanstackQueryApiClient(api);