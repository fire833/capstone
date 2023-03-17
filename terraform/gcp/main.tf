# Primary provider configuration

terraform {

  # Set required version for terraform compliance.
  required_version = "~>1.4"

  required_providers {

    google = {
      source  = "hashicorp/google"
      version = "4.57.0"
    }

    helm = {
      source  = "hashicorp/helm"
      version = "2.9.0"
    }

    random = {
      source  = "hashicorp/random"
      version = "~>3.4.3"
    }

    # tls = {
    #   source  = "hashicorp/tls"
    #   version = "~>4.0.4"
    # }

    # cloudinit = {
    #   source  = "hashicorp/cloudinit"
    #   version = "~>2.2.0"
    # }
  }
}

provider "google" {
    region = var.region
    # access_token = var.access_key
}

provider "helm" {
    
}
