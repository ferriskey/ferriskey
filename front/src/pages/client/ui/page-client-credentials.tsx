import { Client } from "@/api/api.interface";
import { InputText } from "@/components/ui/input-text";
import { Label } from "@/components/ui/label";

export interface PageClientCredentialsProps {
  client: Client
}

export default function PageClientCredentials({ client }: PageClientCredentialsProps) {
  return (
    <div className="flex w-1/3">
      <div className="p-4 border flex flex-col gap-4 rounded-md bg-gray-50 shadow-sm w-full">

        <Label className="text-sm font-medium">
          Client Credentials
        </Label>

        <InputText 
          label="Client ID"
          name="client_id"
          value={client.client_id}
          disabled        
        />
         <InputText 
          label="Client Secret"
          name="client_secret"
          type="password"
          value={client.secret}
          disabled
          togglePasswordVisibility={true}  
        />


      </div>
    </div>
  )
}