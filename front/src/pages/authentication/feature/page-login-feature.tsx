import { useEffect } from "react";
import PageLogin from "../ui/page-login";

export const initiateOAuthLogin = async (realmName: string) => {
  const params = new URLSearchParams({
    response_type: "code",
    client_id: "security-admin-console",
    redirect_uri: "http://localhost:5173/realms/master/authentication/callback", // URL de callback de votre app
    scope: "openid profile email",
    state: crypto.randomUUID(), // Générer un état unique pour la sécurité
  });

  const authUrl = `http://localhost:3333/realms/${realmName}/protocol/openid-connect/auth?${params.toString()}`;
  window.location.href = authUrl;
};

// J'aimerais que si j'ai une erreur de connexion, je puisse afficher un message d'erreur proprement

export default function PageLoginFeature() {
  const handleAuth = async () => {
    await initiateOAuthLogin("master");
  };

  useEffect(() => {
    // Check if required OAuth parameters are missing
    const urlParams = new URLSearchParams(window.location.search);
    const clientId = urlParams.get("client_id");
    const redirectUri = urlParams.get("redirect_uri");

    if (!clientId || !redirectUri) {
      handleAuth();
    }
  }, []);

  return <PageLogin />;
}
