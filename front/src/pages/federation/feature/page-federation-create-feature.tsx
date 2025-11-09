import { RouterParams } from "@/routes/router";
import { useNavigate, useParams } from "react-router";

export default function FederationCreateFeature() {
  const navigate = useNavigate()
  const { realm_name } = useParams<RouterParams>()
}