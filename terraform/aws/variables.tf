
variable "region" {
  description = "Specify the region you want to deploy to."
  default     = "us-east-1"
}

variable "node_count" {
  description = "Specify the default number of nodes to be created."
  default     = 4
}

variable "node_count_max" {
  description = "Specify the maximum number of nodes to be created."
  default = 6
}

variable "cluster_name" {
  description = "Specify the name for your cluster."
  default     = "grid_cluster"
}

variable "cluster_version" {
  description = "Specify the cluster version you want."
  default     = "1.24"
}

# variable "access_key" {
#   description = "Provide access key for authentication"
# }

# variable "secret_access_key" {
#   description = "Provide secret access key for authentication"
# }
