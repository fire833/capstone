provider "helm" {
  kubernetes {
    host                   = data.azurerm_kubernetes_cluster.credentials.kube_config.0.host
    client_certificate     = base64decode(data.azurerm_kubernetes_cluster.credentials.kube_config.0.client_certificate)
    client_key             = base64decode(data.azurerm_kubernetes_cluster.credentials.kube_config.0.client_key)
    cluster_ca_certificate = base64decode(data.azurerm_kubernetes_cluster.credentials.kube_config.0.cluster_ca_certificate)
  }
}


resource "helm_release" "grid-chart-release" {
  name  = "grid-cluster-chart"
  repository = "https://fire833.github.io/capstone/"
  chart = "selenium-grid-cluster"

  version = "0.6.0"

  depends_on = [
    azurerm_kubernetes_cluster.grid-cluster,
    azurerm_public_ip.lb-ip
  ]

  set {
    name = "cloud_provider"
    value = "Azure"
  }

  set {
    name = "cloud_specific_config.Azure.load_balancer_ip"
    value = azurerm_public_ip.lb-ip.ip_address
  }

  set {
    name = "nodes.resources.limits.cpu"
    value = "${tostring(var.selenium_node_cpu_limit)}m"
  }

  set {
    name = "nodes.resources.limits.memory"
    value = "${tostring(var.selenium_node_ram_limit)}Mi"
  }

  set {
    name = "nodes.chrome.maxReplicas"
    value = tostring(var.max_chrome_nodes)
  }

  set {
    name = "nodes.firefox.maxReplicas"
    value = tostring(var.max_firefox_nodes)
  }
  
  set {
    name = "nodes.edge.maxReplicas"
    value = tostring(var.max_edge_nodes)
  }
}


