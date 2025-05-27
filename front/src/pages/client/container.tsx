import { Heading } from "@/components/ui/heading";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useState } from "react";
import { Outlet } from "react-router";
import CreateClientModalFeature from "./feature/create-client-modal-feature";

export default function Container() {
  const [tab, setTab] = useState("list")

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
            <CreateClientModalFeature />
          </div>
        </div>

      </div>

      <Outlet />
    </div>
  )
}