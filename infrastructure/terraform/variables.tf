variable "cluster_name" {
  description = "Nom du cluster k3d"
  default     = "ferriskey"
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
