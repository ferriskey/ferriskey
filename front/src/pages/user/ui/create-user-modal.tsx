import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Dispatch, SetStateAction } from 'react';
import { useFormContext } from "react-hook-form";
import { Fragment } from 'react/jsx-runtime';
import { Switch } from '../../../components/ui/switch';
import { CreateUserSchema } from '../validators';

type Props = {
  realm: string,
  onSubmit: (data: CreateUserSchema) => void
  openState: [boolean, Dispatch<SetStateAction<boolean>>]
}

export default function CreateUserModal(props: Props) {
  const form = useFormContext<CreateUserSchema>()
  const [open, setOpen] = props.openState

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>
          Create User
        </Button>
      </DialogTrigger>

      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Create User</DialogTitle>
          <DialogDescription>
            Create a new user in the selected realm: {props.realm}.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={form.handleSubmit(props.onSubmit)}>
          <div className="grid gap-5 py-4">
            <div className="flex flex-col gap-1">
              <FormField
                control={form.control}
                name="username"
                render={({ field }) => (
                  <Fragment>
                    <FormLabel htmlFor="name" className="text-right">
                      Username
                    </FormLabel>
                    <Input
                      id="username"
                      className="col-span-3"
                      {...field}
                    />
                    <FormMessage />
                  </Fragment>
                )}
              />
            </div>

            <div className="flex flex-col gap-1">
              <FormField
                control={form.control}
                name="firstname"
                render={({ field }) => (
                  <Fragment>
                    <FormLabel htmlFor="firstname" className="text-right">
                      Firstname
                    </FormLabel>
                    <Input
                      id="firstname"
                      className="col-span-3"
                      {...field}
                    />
                    <FormMessage />
                  </Fragment>
                )}
              />
            </div>

            <div className="flex flex-col gap-1">
              <FormField
                control={form.control}
                name="lastname"
                render={({ field }) => (
                  <Fragment>
                    <FormLabel htmlFor="lastname" className="text-right">
                      Lastname
                    </FormLabel>
                    <Input
                      id="lastname"
                      className="col-span-3"
                      {...field}
                    />
                    <FormMessage />
                  </Fragment>
                )}
              />
            </div>

            <div className="flex flex-col gap-1">
              <FormField
                control={form.control}
                name="email"
                render={({ field }) => (
                  <Fragment>
                    <FormLabel htmlFor="email" className="text-right">
                      Email
                    </FormLabel>
                    <Input
                      id="email"
                      className="col-span-3"
                      {...field}
                    />
                    <FormMessage />
                  </Fragment>
                )}
              />
            </div>


            <div className="flex flex-col gap-1">
              <FormField
                control={form.control}
                name="email_verified"
                render={({ field }) => (
                  <FormItem className="flex flex-row items-center justify-between gap-5 rounded-lg border p-3 shadow-sm">
                    <div className="space-y-0.5">
                      <FormLabel>Verified email</FormLabel>
                      <FormDescription>
                        Choose between verified and unverified email as default status.
                      </FormDescription>
                    </div>
                    <FormControl>
                      <Switch
                        checked={field.value}
                        onCheckedChange={field.onChange}
                      />
                    </FormControl>
                  </FormItem>
                )}
              />
            </div>
          </div>
          <DialogFooter>
            <Button type="submit">Save changes</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}