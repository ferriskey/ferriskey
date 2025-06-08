import { Heading } from "@/components/ui/heading";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useState } from "react";
import { Outlet, useNavigate, useParams } from 'react-router'
import { CLIENT_CREATE_URL, CLIENTS_URL } from '@/routes/sub-router/client.router.ts'
import { RouterParams } from '@/routes/router.ts'
import { Button } from '@/components/ui/button.tsx'

export default function Container() {
  const { realm_name } = useParams<RouterParams>()
  const [tab, setTab] = useState("list")
  const navigate = useNavigate()

  const handleCreateClient = () => {
    navigate(`${CLIENTS_URL(realm_name)}${CLIENT_CREATE_URL}`)
  }

  return (
    <div className="flex flex-col gap-4 p-8">
      <div className="flex flex-col gap-2 border-b pb-4">
        <div className="flex flex-col gap-2">
          <Heading>Clients</Heading>
          <p>Manage your clients effectively</p>
        </div>
        <div className="flex justify-between items-center">
          <Tabs defaultValue={tab} onValueChange={setTab}>
            <TabsList>
              <TabsTrigger value={"list"}>Clients list</TabsTrigger>
            </TabsList>
          </Tabs>


          <div>
            <Button
              variant="outline"
              onClick={handleCreateClient}
            >
              Create Client
            </Button>

          </div>
        </div>

      </div>

      <Outlet />
    </div>
  )
}
