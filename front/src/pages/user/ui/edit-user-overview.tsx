import { useFormContext } from 'react-hook-form';
import { Button } from '../../../components/ui/button';
import { FormControl, FormField, FormItem, FormLabel, FormMessage } from '../../../components/ui/form';
import { Input } from '../../../components/ui/input';
import { FormSwitch } from '../../../components/ui/switch';
import { UpdateUserSchema } from '../validators';

type Props = {
  onSubmit: (data: UpdateUserSchema) => void
}

export default function EditUserOverview(props: Props) {
  const form = useFormContext<UpdateUserSchema>()

  return (
    <div className="max-w-2xl p-5">
      <form onSubmit={form.handleSubmit(props.onSubmit)} className="space-y-6">
        <div className="flex flex-col gap-6">
          <FormField
            control={form.control}
            name="username"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Username</FormLabel>
                <FormControl>
                  <Input placeholder="Enter username" disabled {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="email"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Email</FormLabel>
                <FormControl>
                  <Input type="email" placeholder="Enter email" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="firstname"
            render={({ field }) => (
              <FormItem>
                <FormLabel>First Name</FormLabel>
                <FormControl>
                  <Input placeholder="Enter first name" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="lastname"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Last Name</FormLabel>
                <FormControl>
                  <Input placeholder="Enter last name" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
        </div>

        <div className="flex flex-col gap-1">
          <FormField
            control={form.control}
            name="enabled"
            render={({ field }) => (
              <FormSwitch
                label="User Enabled"
                description="Choose between enabled and disabled user as default status."
                checked={field.value}
                onChange={field.onChange}
              />
            )}
          />
        </div>

        <div className="flex flex-col gap-1">
          <FormField
            control={form.control}
            name="email_verified"
            render={({ field }) => (
              <FormSwitch
                label="Email Verified"
                description={(value) => value ? "Email is verified" : "Email is not verified"}
                checked={field.value}
                onChange={field.onChange}
              />
            )}
          />
        </div>

        <div className="flex justify-end space-x-3 pt-6 border-t border-gray-200">
          <Button type="submit">
            Save Changes
          </Button>
        </div>
      </form>
    </div>
  )
}