import BlockContent from "@/components/ui/block-content";
import { FormField } from "@/components/ui/form";
import { InputText } from "@/components/ui/input-text";
import { UpdateRealmSchema } from "../validators";
import { useFormContext } from "react-hook-form";
import { Realm, SigningAlgorithm } from "@/api/core.interface";
import { Select, SelectTrigger, SelectValue } from "@/components/ui/select";
import { SelectContent, SelectItem } from "@radix-ui/react-select";

type Props = {
  hasChanges: boolean;
  realm: Realm;
}


export default function PageRealmSettingsGeneral({ realm }: Props) {
  const form = useFormContext<UpdateRealmSchema>();
  return (<div className="w-full">
    <BlockContent title="General settings">
      <div className="flex flex-col gap-3">
        <InputText label="Realm ID" value={realm.id} disabled={true} name="id" />

        <FormField
          control={form.control}
          name="name"
          render={({ field }) => <InputText label="Name" {...field} />}
        />

        <FormField
          control={form.control}
          name="default_signing_algorithm"
          render={({ field }) => (
            <Select
              onValueChange={(value) => field.onChange(value)}
              value={field.value}
            >
              <SelectTrigger>
                <SelectValue>{field.value}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                {
                  Object.values(SigningAlgorithm).map((value) => {
                    return (
                      <SelectItem value={value}>{value.toString()}</SelectItem>
                    )
                  })
                }
              </SelectContent>
            </Select>
          )}
        />
      </div>
    </BlockContent>
  </div>)
}
