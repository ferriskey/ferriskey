import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { findBareImports } from './bundle-imports.ts'

const distDirectory = process.argv[2] ?? 'dist'

const chunks = readdirSync(distDirectory, { recursive: true, encoding: 'utf8' }).filter((file) =>
  file.endsWith('.js')
)

let failures = 0

for (const chunk of chunks) {
  const bare = findBareImports(readFileSync(join(distDirectory, chunk), 'utf8'))
  for (const specifier of bare) {
    failures += 1
    console.error(`error  ${chunk}  imports "${specifier}", which a browser cannot resolve`)
  }
}

if (failures > 0) {
  console.error(`\n${failures} bare import(s) in ${distDirectory}`)
  process.exit(1)
}

console.log(`${chunks.length} chunks in ${distDirectory} have no bare import`)
