# Primary provider configuration

terraform {

  # Set required version for terraform compliance.
  required_version = "~>1.4"

  required_providers {

    aws = {
      source  = "hashicorp/aws"
      version = "4.59.0"
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

provider "aws" {
  region = var.region
  # access_key = var.access_key
  # secret_key = var.secret_access_key
}

provider "helm" {
  kubernetes {
    host                   = data.aws_eks_cluster.eks.endpoint
    token                  = data.aws_eks_cluster_auth.eks.token
    cluster_ca_certificate = base64decode(data.aws_eks_cluster.eks.certificate_authority[0].data)
  }
}
