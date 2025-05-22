import { Heading } from "@/components/ui/heading";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { RouterParams } from "@/routes/router";
import { useState } from "react";
import { Outlet, useParams } from "react-router";
import CreateUserModalFeature from "./feature/create-user-modal-feature";

export default function Container() {
  const { realm_name } = useParams<RouterParams>()
  const [tab, setTab] = useState("list")
  return (
    <div className="flex flex-col gap-4 p-8">
      <div className="flex flex-col gap-2 border-b pb-4">
        <div className="flex flex-col gap-2">
          <Heading>Users</Heading>
          <p>Manage users in {realm_name}</p>
        </div>
        <div className="flex justify-between items-center">
          <Tabs defaultValue={tab} onValueChange={setTab}>
            <TabsList>
              <TabsTrigger value={"list"}>Users list</TabsTrigger>
            </TabsList>
          </Tabs>
          <div>
            <CreateUserModalFeature realm={realm_name ?? 'master'} />
          </div>
        </div>
      </div>

      <Outlet />
    </div>
  )
}