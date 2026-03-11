import { UseFormReturn } from 'react-hook-form'
import { PasswordPolicySchema } from '../feature/page-realm-settings-password-policy-feature'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
import { Switch } from '@/components/ui/switch'
import { Input } from '@/components/ui/input'
import FloatingActionBar from '@/components/ui/floating-action-bar'

export interface PageRealmSettingsPasswordPolicyProps {
    form: UseFormReturn<PasswordPolicySchema>
    hasChanges: boolean
    handleSubmit: (values: PasswordPolicySchema) => void
}

export default function PageRealmSettingsPasswordPolicy({ form, hasChanges, handleSubmit }: PageRealmSettingsPasswordPolicyProps) {
    return (
        <Form {...form}>
            <div className='flex flex-col gap-8'>
                <div className='flex flex-col gap-1'>
                    <div className='mb-4'>
                        <p className='text-xs text-muted-foreground mb-0.5'>Configure realm-wide password constraints</p>
                        <h2 className='text-base font-semibold'>Password Policy</h2>
                    </div>

                    <FormField
                        control={form.control}
                        name='minLength'
                        render={({ field }) => (
                            <div className='flex items-center justify-between py-4 border-t'>
                                <div className='w-1/3'>
                                    <p className='text-sm font-medium'>Minimum Length</p>
                                    <p className='text-sm text-muted-foreground mt-0.5'>The minimum number of characters required for a password.</p>
                                </div>
                                <div className='w-1/2'>
                                    <FormItem className='max-w-[120px]'>
                                        <FormControl>
                                            <Input type='number' {...field} onChange={e => field.onChange(parseInt(e.target.value) || 0)} />
                                        </FormControl>
                                        <FormMessage />
                                    </FormItem>
                                </div>
                            </div>
                        )}
                    />

                    <FormField
                        control={form.control}
                        name='requireUppercase'
                        render={({ field }) => (
                            <div className='flex items-center justify-between py-4 border-t'>
                                <div className='w-1/3'>
                                    <p className='text-sm font-medium'>Require Uppercase</p>
                                    <p className='text-sm text-muted-foreground mt-0.5'>Require at least one uppercase letter (A-Z).</p>
                                </div>
                                <div className='w-1/2'>
                                    <FormItem className='flex flex-row items-center gap-3'>
                                        <FormControl>
                                            <Switch checked={field.value} onCheckedChange={field.onChange} />
                                        </FormControl>
                                        <FormLabel className='!mt-0 font-normal text-muted-foreground'>
                                            {field.value ? 'Enabled' : 'Disabled'}
                                        </FormLabel>
                                    </FormItem>
                                </div>
                            </div>
                        )}
                    />

                    <FormField
                        control={form.control}
                        name='requireLowercase'
                        render={({ field }) => (
                            <div className='flex items-center justify-between py-4 border-t'>
                                <div className='w-1/3'>
                                    <p className='text-sm font-medium'>Require Lowercase</p>
                                    <p className='text-sm text-muted-foreground mt-0.5'>Require at least one lowercase letter (a-z).</p>
                                </div>
                                <div className='w-1/2'>
                                    <FormItem className='flex flex-row items-center gap-3'>
                                        <FormControl>
                                            <Switch checked={field.value} onCheckedChange={field.onChange} />
                                        </FormControl>
                                        <FormLabel className='!mt-0 font-normal text-muted-foreground'>
                                            {field.value ? 'Enabled' : 'Disabled'}
                                        </FormLabel>
                                    </FormItem>
                                </div>
                            </div>
                        )}
                    />

                    <FormField
                        control={form.control}
                        name='requireNumber'
                        render={({ field }) => (
                            <div className='flex items-center justify-between py-4 border-t'>
                                <div className='w-1/3'>
                                    <p className='text-sm font-medium'>Require Number</p>
                                    <p className='text-sm text-muted-foreground mt-0.5'>Require at least one digit (0-9).</p>
                                </div>
                                <div className='w-1/2'>
                                    <FormItem className='flex flex-row items-center gap-3'>
                                        <FormControl>
                                            <Switch checked={field.value} onCheckedChange={field.onChange} />
                                        </FormControl>
                                        <FormLabel className='!mt-0 font-normal text-muted-foreground'>
                                            {field.value ? 'Enabled' : 'Disabled'}
                                        </FormLabel>
                                    </FormItem>
                                </div>
                            </div>
                        )}
                    />

                    <FormField
                        control={form.control}
                        name='requireSpecial'
                        render={({ field }) => (
                            <div className='flex items-center justify-between py-4 border-t'>
                                <div className='w-1/3'>
                                    <p className='text-sm font-medium'>Require Special Character</p>
                                    <p className='text-sm text-muted-foreground mt-0.5'>Require at least one special character (e.g., !@#$%^&*).</p>
                                </div>
                                <div className='w-1/2'>
                                    <FormItem className='flex flex-row items-center gap-3'>
                                        <FormControl>
                                            <Switch checked={field.value} onCheckedChange={field.onChange} />
                                        </FormControl>
                                        <FormLabel className='!mt-0 font-normal text-muted-foreground'>
                                            {field.value ? 'Enabled' : 'Disabled'}
                                        </FormLabel>
                                    </FormItem>
                                </div>
                            </div>
                        )}
                    />

                    <FormField
                        control={form.control}
                        name='maxAgeDays'
                        render={({ field }) => (
                            <div className='flex items-center justify-between py-4 border-t'>
                                <div className='w-1/3'>
                                    <p className='text-sm font-medium'>Password Max Age (Days)</p>
                                    <p className='text-sm text-muted-foreground mt-0.5'>Number of days before a password must be changed. Leave empty for no limit.</p>
                                </div>
                                <div className='w-1/2'>
                                    <FormItem className='max-w-[120px]'>
                                        <FormControl>
                                            <Input
                                                type='number'
                                                value={field.value || ''}
                                                onChange={e => field.onChange(e.target.value === '' ? null : parseInt(e.target.value))}
                                            />
                                        </FormControl>
                                        <FormMessage />
                                    </FormItem>
                                </div>
                            </div>
                        )}
                    />
                </div>

                <FloatingActionBar
                    show={hasChanges}
                    title='Save Changes'
                    actions={[{ label: 'Save', variant: 'default', onClick: () => form.handleSubmit(handleSubmit)() }]}
                    description='You have unsaved changes in your password policy settings.'
                    onCancel={form.reset}
                />
            </div>
        </Form>
    )
}
