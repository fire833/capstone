# Copyright (c) HashiCorp, Inc.
# SPDX-License-Identifier: MPL-2.0

variable "appId" {
  description = "Azure Kubernetes Service Cluster service principal"
}

variable "password" {
  description = "Azure Kubernetes Service Cluster password"
}

variable "cluster_name" {
  description = "Specify the name to use when creating the AKS cluster. Must only use URL-safe characters"
  default = "aks-grid-cluster"
}

variable "cluster_region" {
  description = "Specify the region in which to create the AKS cluster"
  default = "West US 3"
}

variable "vm_size" {
  description = "Specify the azure VM size to create when provisioning VMs"
  default = "Standard_F8s_v2"
}

variable "initial_nodes" {
  description = "Specify the number of nodes which the cluster will be created with"
  default = 1
}

variable "max_nodes" {
  description = "Specify the maximum number of nodes which the cluster may scale to"
  default = 8
}
variable "min_nodes" {
  description = "Specify the minimum number of nodes which the cluster may scale down to"
  default = 1
}


variable "max_chrome_nodes" {
  description = "Specify the maximum number of selenium nodes which can be provisioned"
  default = 20
}

variable "min_chrome_nodes" {
  description = "Specify the minimum number of selenium nodes which can be provisioned"
  default = 1
}

variable "max_firefox_nodes" {
  description = "Specify the maximum number of selenium nodes which can be provisioned"
  default = 20
}

variable "min_firefox_nodes" {
  description = "Specify the minimum number of selenium nodes which can be provisioned"
  default = 1
}

variable "max_edge_nodes" {
  description = "Specify the maximum number of selenium nodes which can be provisioned"
  default = 20
}

variable "min_edge_nodes" {
  description = "Specify the minimum number of selenium nodes which can be provisioned"
  default = 1
}

variable "selenium_node_cpu_limit" {
  description = "Specify how many milli CPU cores a selenium node may use."
  default = 1000
}

variable "selenium_node_ram_limit" {
  description = "Specify how many megabytes of RAM a selenium node may use."
  default = 1000
}

variable "helm_values" {
  default = ""
}