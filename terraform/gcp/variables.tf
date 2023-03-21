# Specify variables for granular configuration.

variable "region" {
  description = "Specify the region you want to deploy to. This may also be a zone name to create a zonal cluster."
  default     = "us-east1"
}

variable "project_id" {
  description = "Specify the project you want to associate with this deployment."
}

variable "node_count" {
  description = "Specify the default number of nodes to be created. If this cluster is regional, this is the number of nodes per zone (default 3, so you will provision 3x this number of nodes). If this is a zonal cluster (a zone was given to the region variable), you will provision exactly this many nodes"
  default = 1
}

variable "node_type" {
  description = "Specify the GCP VM type for nodes in the initial node pool"
  default = "e2-medium"
}

variable "cluster_name" {
  description = "Specify the name for your cluster."
  default = "grid-cluster"
}

variable "cluster_version" {
  description = "Specify the cluster version you want."
  default     = "1.24"
}

variable "base_autoscaling_cpu" {
  description = "Specify the number of CPU cores which you expect the default node pool to consume, otherwise the default node pool utilization will count against the autoscaling limits"
  default = 4
}

variable "base_autoscaling_ram" {
  description = "Specify the gigabytes of RAM which you expect the default node pool to consume, otherwise the default node pool utilization will count against the autoscaling limits"
  default = 4
}

variable "max_chrome_nodes" {
  description = "Specify the maximum number of selenium nodes which can be provisioned"
  default = 10
}


variable "max_firefox_nodes" {
  description = "Specify the maximum number of selenium nodes which can be provisioned"
  default = 10
}

variable "max_edge_nodes" {
  description = "Specify the maximum number of selenium nodes which can be provisioned"
  default = 10
}

variable "selenium_node_cpu_limit" {
  description = "Specify how many milli CPU cores a selenium node may use."
  default = 900
}

variable "selenium_node_ram_limit" {
  description = "Specify how many megabytes of RAM a selenium node may use."
  default = 750
}


locals {
  max_selenium_nodes = var.max_chrome_nodes + var.max_firefox_nodes + var.max_edge_nodes
  cluster_autoscaling_max_cpu_cores = var.base_autoscaling_cpu + ceil(local.max_selenium_nodes * (var.selenium_node_cpu_limit / 1000))
  cluster_autoscaling_max_gb_ram    = var.base_autoscaling_ram + ceil(local.max_selenium_nodes * (var.selenium_node_ram_limit / 1000))
}
