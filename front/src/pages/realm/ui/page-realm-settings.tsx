import { Heading } from "lucide-react";

interface PageRealmSettingsProps {
  realm_name: string
}

export default function PageRealmSettings({ realm_name} : PageRealmSettingsProps) {
  return (
    <div className="flex flex-col gap-4 p-8">
      <div className="flex flex-col gap-2 border-b pb-4">
        <div className="flex flex-col gap-2">
          <Heading>{realm_name}</Heading>
        </div>
      </div>
    </div>
  )
}
