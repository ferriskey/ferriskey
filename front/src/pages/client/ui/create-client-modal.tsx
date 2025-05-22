import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { CreateClientSchema } from "../feature/create-client-modal-feature";
import { Form, UseFormReturn } from "react-hook-form";
import { FormField } from "@/components/ui/form";
import { Switch } from "@/components/ui/switch";

export interface CreateClientModalProps {
  form: UseFormReturn<CreateClientSchema>
  onSubmit: () => void
}

export default function CreateClientModal({ form, onSubmit }: CreateClientModalProps) {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline">
          Create Client
        </Button>
      </DialogTrigger>

      <DialogContent className="sm:max-w-[425px] lg:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Create Client</DialogTitle>
          <DialogDescription>
            Clients are applications and services that can request authentication of a user.
          </DialogDescription>
        </DialogHeader>
        <Form {...form}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="name" className="text-right">
                Client ID
              </Label>
              <FormField 
                control={form.control}
                name="clientId"
                render={({ field }) => (
                  <Input id="clientId" {...field} className="col-span-3" />
                )}
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="username" className="text-right">
                Name
              </Label>
              <FormField 
                control={form.control}
                name="name"
                render={({ field }) => (
                  <Input id="name" {...field} className="col-span-3" />
                )}
              />
            </div>

            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="enabled" className="text-right">
                Enabled
              </Label>
              <FormField
                control={form.control}
                name="enabled"
                render={({ field }) => (
                  <Switch checked={field.value} onCheckedChange={field.onChange} id="enabled" />
                )}
              />
            </div>

            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="publicClient" className="text-left">
                Client Authentication
              </Label>
              <FormField
                control={form.control}
                name="clientAuthentication"
                render={({ field }) => (
                  <Switch checked={field.value} onCheckedChange={field.onChange} id="publicClient" />
                )}
              />
            </div>
          </div>
          <DialogFooter>
            <Button onClick={onSubmit}>Save changes</Button>
          </DialogFooter>

        </Form>
        
      </DialogContent>
    </Dialog>
  )
}