import { useForm } from "react-hook-form";
import { UpdateRealmSchema, updateRealmValidator } from "../validators";
import { zodResolver } from "@hookform/resolvers/zod";
import { SigningAlgorithm } from "@/api/core.interface";
import { Form } from "@/components/ui/form";
import PageRealmSettingsGeneral from "../ui/page-realm-settings-general";
import useRealmStore from "@/store/realm.store";
import { mapRealms } from "@/api/core.mapper";

export default function PageRealmSettingsGeneralFeature() {
  const { userRealms } = useRealmStore();

  const realm = mapRealms(userRealms).find((item) => item.name === 'master');

  const form = useForm<UpdateRealmSchema>({
    resolver: zodResolver(updateRealmValidator),
    mode: 'all',
    values: {
      name: realm?.name ?? 'master',
      default_signing_algorithm: SigningAlgorithm.RS256,
    }
  })


  if (!realm) return null;

  return (
    <Form {...form}>
      <PageRealmSettingsGeneral realm={realm} hasChanges={false} />
    </Form>
  )
}
