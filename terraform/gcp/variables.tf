# Specify variables for granular configuration.

variable "region" {
  description = "Specify the region you want to deploy to."
  default     = "us-central1"
}

variable "project" {
  description = "Specify the project you want to associate with this deployment."
}

variable "node_count" {
  description = "Specify the default number of nodes to be created."
  default = 4
}

variable "cluster_name" {
  description = "Specify the name for your cluster."
  default = "grid_cluster"
}

variable "cluster_version" {
  description = "Specify the cluster version you want."
  default = "1.24"
}

# variable "access_key" {
#   description = "Provide access key for authentication"
# }

