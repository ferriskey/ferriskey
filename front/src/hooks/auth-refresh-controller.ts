const DEFAULT_REFRESH_BACKOFF_MS = 15_000

type RefreshControllerState<T> = {
  blockedUntil: number
  inFlight: Promise<T> | null
}

function getState<T>(
  states: Map<string, RefreshControllerState<T>>,
  key: string
) {
  let state = states.get(key)

  if (!state) {
    state = {
      blockedUntil: 0,
      inFlight: null,
    }
    states.set(key, state)
  }

  return state
}

export function createRefreshController(
  now: () => number = () => Date.now(),
  backoffMs: number = DEFAULT_REFRESH_BACKOFF_MS
) {
  const states = new Map<string, RefreshControllerState<unknown>>()

  return {
    async run<T>(key: string, factory: () => Promise<T>) {
      const state = getState(states, key)

      if (state.inFlight) {
        return state.inFlight as Promise<T>
      }

      if (state.blockedUntil > now()) {
        throw new Error('refresh temporarily blocked')
      }

      const promise = factory()
        .then((result) => {
          state.blockedUntil = 0
          return result
        })
        .catch((error) => {
          state.blockedUntil = now() + backoffMs
          throw error
        })
        .finally(() => {
          state.inFlight = null
        })

      state.inFlight = promise as Promise<unknown>

      return promise
    },
    reset(key: string) {
      states.delete(key)
    },
  }
}

export const authRefreshController = createRefreshController()
