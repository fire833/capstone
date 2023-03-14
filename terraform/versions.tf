variable "gke_projectid" {
  description = "GKE project ID"
}

terraform {
    required_providers {
      google = {
        source = "hashicorp/google"
        project = var.gke_projectid
        version = "4.27.0"
      }
    }

    required_version = ">= 0.14"
}