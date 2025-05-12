variable "cluster_name" {
  description = "Nom du cluster k3d"
  default     = "gke_nathael-dev_europe-west1_onepiece"
}

variable "grafana_url" {
  description = "URL d'accès à Grafana"
  default     = "http://localhost:3000"
}

variable "grafana_admin_user" {
  description = "Utilisateur Grafana"
  default     = "admin"
}

variable "grafana_admin_password" {
  description = "Mot de passe Grafana"
  default     = "admin"
}
