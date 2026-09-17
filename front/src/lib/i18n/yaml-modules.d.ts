declare module '*.yaml' {
  type CatalogNode = string | { [key: string]: CatalogNode }

  const catalog: { [key: string]: CatalogNode }

  export default catalog
}
