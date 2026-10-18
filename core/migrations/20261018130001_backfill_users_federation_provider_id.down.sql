UPDATE users SET federation_provider_id = NULL WHERE federation_provider_id IS NOT NULL;
