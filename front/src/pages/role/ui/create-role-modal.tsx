import { UseFormReturn } from "react-hook-form";
import { CreateRoleSchema } from "../schemas/create-role.schema";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { InputText } from "@/components/ui/input-text";
import { Permissions } from "@/api/api.interface";

export interface CreateRoleModalProps {
  form: UseFormReturn<CreateRoleSchema>
  selectedPermissions: Permissions[]
}

export default function CreateRoleModal({}: CreateRoleModalProps) {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button>
          Create Role
        </Button>
      </DialogTrigger>

      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Create Role</DialogTitle>
          <DialogDescription>
            Fill in the details to create a new role.
          </DialogDescription>
        </DialogHeader>

        <div>
          <div className="flex flex-col gap-4">
            <InputText 
              name="name"
              label="Name"
            />

            <InputText 
              name="description"
              label="Description"
            />
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}