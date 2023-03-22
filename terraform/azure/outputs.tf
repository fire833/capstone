# Copyright (c) HashiCorp, Inc.
# SPDX-License-Identifier: MPL-2.0

output "resource_group_name" {
  value = azurerm_resource_group.grid-resource-group.name
}

output "kubernetes_cluster_name" {
  value = azurerm_kubernetes_cluster.grid-cluster.name
}

output "host" {
  value = azurerm_kubernetes_cluster.grid-cluster.kube_config.0.host
  sensitive = true
}

output "hub_endpoint" {
  value = "http://${azurerm_public_ip.lb-ip.ip_address}:9994/"
}

# output "cluster-node-rg" {
#   value = azurerm_kubernetes_cluster.grid-cluster.node_resource_group
# }