
variable "region" {
  description = "Specify the region you want to deploy to."
  default     = "us-east-1"
}

variable "node_count" {
  description = "Specify the default number of nodes to be created."
  default     = 2
}

variable "node_count_max" {
  description = "Specify the maximum number of nodes to be created."
  default     = 8
}

variable "cluster_name" {
  description = "Specify the name for your cluster."
  default     = "grid_cluster"
}

variable "cluster_version" {
  description = "Specify the cluster version you want."
  default     = "1.24"
}

variable "deploy_namespace" {
  description = "Specify the Kubernetes namespace to deploy to."
  default     = "default"
}


variable "max_chrome_nodes" {
  description = "Specify the maximum number of selenium nodes which can be provisioned"
  default = 20
}


variable "max_firefox_nodes" {
  description = "Specify the maximum number of selenium nodes which can be provisioned"
  default = 20
}

variable "max_edge_nodes" {
  description = "Specify the maximum number of selenium nodes which can be provisioned"
  default = 20
}

variable "selenium_node_cpu_limit" {
  description = "Specify how many milli CPU cores a selenium node may use."
  default = 1000
}

variable "selenium_node_ram_limit" {
  description = "Specify how many megabytes of RAM a selenium node may use."
  default = 1500
}


variable "instance_type" {
  description = "Specify the instance type to use for EC2 nodes in the cluster"
  default = "c5a.2xlarge"
}

variable "helm_values" {
  default = ""
}

# variable "access_key" {
#   description = "Provide access key for authentication"
# }

# variable "secret_access_key" {
#   description = "Provide secret access key for authentication"
# }
