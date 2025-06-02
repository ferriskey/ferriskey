import { Client } from "@/api/api.interface";
import { Heading } from "@/components/ui/heading";
import { InputText } from "@/components/ui/input-text";

export interface PageClientSettingsProps {
  client: Client
}

export default function PageClientSettings({ client }: PageClientSettingsProps) {
  return (
    <div>
      <div className="flex flex-col gap-4">
        <Heading size={4} className="text-gray-900">General Settings</Heading>

        <div className="flex w-1/3">
          <div className="flex flex-col gap-4 p-4 bg-gray-50 border rounded-md w-full">
            <div className="flex flex-col gap-2">
              <InputText 
                name="client_id"
                label="Client ID"
                value={client.client_id}
                disabled
              />
            </div>

            <div className="flex flex-col gap-2">
              <InputText 
                name="client_name"
                label="Client Name"
                value={client.name}
                disabled
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}