import { useState } from 'react'
import './app.css'
import { useDispatch, useSelector } from 'react-redux'
import { AppDispatch } from '@/store/store'
import { getUserState } from '@/store/user.store'
import LoaderSpinner from '@/components/ui/loader-spinner'

function App() {
  const dispatch = useDispatch<AppDispatch>()
  const { isAuthenticated, isLoading, token, user } = useSelector(getUserState)

  if (isLoading) {
    return (
      <div className='flex h-screen w-screen items-center justify-center'>
        <LoaderSpinner />
      </div>
    )
  }

  return (
    <>
      <div>hello FerrisKey</div>
    </>
  )
}

export default App
