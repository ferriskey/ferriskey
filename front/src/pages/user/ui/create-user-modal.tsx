import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { UseFormReturn } from "react-hook-form";
import { CreateUserSchema } from "../feature/create-user-modal-feature";
import {  FormField } from "@/components/ui/form";

export interface CreateUserModalProps {
  form: UseFormReturn<CreateUserSchema>
  onSubmit: (data: CreateUserSchema) => void
}

export default function CreateUserModal({ form, onSubmit }: CreateUserModalProps) {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button>
          Create User
        </Button>
      </DialogTrigger>

      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Edit profile</DialogTitle>
          <DialogDescription>
            Make changes to your profile here. Click save when you're done.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="name" className="text-right">
              Username
            </Label>
            <FormField 
              control={form.control}
              name="username"
              render={({ field }) => (
                <Input id="username" {...field} className="col-span-3" />
              )}
            />
          </div>

          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="email" className="text-right">
              Email
            </Label>
            <FormField 
              control={form.control}
              name="email"
              render={({ field }) => (
                <Input id="email" {...field} className="col-span-3" />
              )}
            />
          </div>

          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="firstName" className="text-right">
              First Name
            </Label>
            <FormField 
              control={form.control}
              name="firstName"
              render={({ field }) => (
                <Input id="firstName" {...field} className="col-span-3" />
              )}
            />
          </div>

          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="firstName" className="text-right">
              First Name
            </Label>
            <FormField 
              control={form.control}
              name="firstName"
              render={({ field }) => (
                <Input id="firstName" {...field} className="col-span-3" />
              )}
            />
          </div>

          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="lastName" className="text-right">
              Last Name
            </Label>
            <FormField 
              control={form.control}
              name="lastName"
              render={({ field }) => (
                <Input id="lastName" {...field} className="col-span-3" />
              )}
            />
          </div>
        </div>
        <DialogFooter>
          <Button onClick={() => onSubmit(form.getValues())}>Save changes</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}