import { Client } from "@/api/api.interface";
import { Heading } from "@/components/ui/heading";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export interface PageClientOverviewProps {
  client: Client
}

export default function PageClientOverview({ client }: PageClientOverviewProps) {
  return (
    <div>
      <div className="flex flex-col gap-4">
        <Heading size={4} className="text-gray-900">General Settings</Heading>

        <div>
          <div className="flex flex-col gap-2">
            <Label className="text-gray-700">Client ID</Label>
            <Input
              className="mt-1"
              placeholder="Enter client ID"
              value={client.client_id}
              disabled
            />
          </div>
        </div>
      </div>
    </div>
  )
}