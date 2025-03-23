import { UserState } from '@/contracts/states.interface'
import { createSlice } from '@reduxjs/toolkit'
import { RootState } from './store'

const USER_KEY = 'user'

export const initialUserState: UserState = {
  isAuthenticated: false,
  isLoading: true,
  token: null,
  user: null,
}

export const userSlice = createSlice({
  name: USER_KEY,
  initialState: initialUserState,
  reducers: {
    switchIsLoading: (state, action) => {
      state.isLoading = action.payload
    },
  },
})

export const userReducer = userSlice.reducer
export const userActions = userSlice.actions

export const getUserState = (root: RootState) => root[USER_KEY]
