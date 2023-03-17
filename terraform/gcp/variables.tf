# Specify variables for granular configuration.

variable "region" {
  description = "Specify the region you want to deploy to. This may also be a zone name to create a zonal cluster."
  default     = "us-central1"
}

variable "project_id" {
  description = "Specify the project you want to associate with this deployment."
}

variable "node_count" {
  description = "Specify the default number of nodes to be created. If this cluster is regional, this is the number of nodes per zone (default 3, so you will provision 3x this number of nodes). If this is a zonal cluster (a zone was given to the region variable), you will provision exactly this many nodes"
  default = 1
}

variable "cluster_name" {
  description = "Specify the name for your cluster."
  default = "grid-cluster"
}

variable "cluster_version" {
  description = "Specify the cluster version you want."
  default     = "1.24"
}

# variable "access_key" {
#   description = "Provide access key for authentication"
# }

