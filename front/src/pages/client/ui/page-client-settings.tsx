import { Client } from "@/api/api.interface";
import { Heading } from "@/components/ui/heading";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

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
              <Label className="text-gray-700">Client ID</Label>
              <Input
                className="bg-white"
                placeholder="Enter client ID"
                value={client.client_id}
                disabled
              />
            </div>

            <div className="flex flex-col gap-2">
              <Label className="text-gray-700">Client Name</Label>
              <Input
                className="bg-white"
                placeholder="Enter client name"
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