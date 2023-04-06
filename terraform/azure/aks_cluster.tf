# Copyright (c) HashiCorp, Inc.
# SPDX-License-Identifier: MPL-2.0

provider "azurerm" {
  features {}
}

resource "azurerm_resource_group" "grid-resource-group" {
  name     = "${var.cluster_name}-rg"
  location = var.cluster_region

  tags = {
    environment = "${var.cluster_name}"
  }
}

# Create a fixed, public ip for the load balancer
resource "azurerm_public_ip" "lb-ip" {
  name                = "${var.cluster_name}-loadbalancer-ip"
  resource_group_name = azurerm_kubernetes_cluster.grid-cluster.node_resource_group
  location            = azurerm_kubernetes_cluster.grid-cluster.location
  allocation_method   = "Static"
  sku = "Standard"

  tags = {
    environment = "${var.cluster_name}"
  }
}

resource "azurerm_kubernetes_cluster" "grid-cluster" {
  name                = var.cluster_name
  location            = azurerm_resource_group.grid-resource-group.location
  resource_group_name = azurerm_resource_group.grid-resource-group.name
  dns_prefix          = "${var.cluster_name}-k8s"

  default_node_pool {
    name                = "default"
    node_count          = var.initial_nodes
    max_count           = var.max_nodes
    min_count           = var.min_nodes
    vm_size             = "${var.vm_size}"
    os_disk_size_gb     = 30
    enable_auto_scaling = true
  }

  service_principal {
    client_id     = var.appId
    client_secret = var.password
  }

  role_based_access_control_enabled = true

  tags = {
    environment = "${var.cluster_name}"
  }
}

data "azurerm_kubernetes_cluster" "credentials" {
  name                = azurerm_kubernetes_cluster.grid-cluster.name
  resource_group_name = azurerm_resource_group.grid-resource-group.name
}
