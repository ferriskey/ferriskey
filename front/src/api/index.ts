import axios from 'axios'


export const apiUrl = import.meta.env.VITE_API_URL ?? 'http://localhost:3333'

export interface BaseQuery {
  realm?: string
}

const defaultHeaders = {
  'Content-Type': 'application/json',
}

export const apiClient = axios.create({
  baseURL: apiUrl,
  headers: defaultHeaders,
  withCredentials: true,
})
