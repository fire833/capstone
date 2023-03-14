terraform {
    required_providers {
      google = {
        source = "hashicorp/google"
        version = "4.27.0"
      }
      
      helm = {
        source = "hashicorp/helm"
        version = ">= 3.3.2"
      }
    }

    required_version = ">= 0.14"
}