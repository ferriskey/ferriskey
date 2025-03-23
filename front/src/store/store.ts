import { combineReducers, configureStore } from '@reduxjs/toolkit'
import { userReducer } from './user.store'

export const rootReducer = combineReducers({
  user: userReducer,
})

export function setupStore(preloadedState?: never) {
  return configureStore({
    reducer: rootReducer,
    preloadedState,
  })
}

export type RootState = ReturnType<typeof rootReducer>
export type AppStore = ReturnType<typeof setupStore>
export type AppDispatch = AppStore['dispatch']
