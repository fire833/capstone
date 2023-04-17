resource "helm_release" "grid-chart-release" {
  name  = "grid-cluster-chart"
  repository = "https://fire833.github.io/capstone/"
  chart = "selenium-grid-cluster"

  depends_on = [
    google_container_cluster.primary
  ]

  set {
    name = "cloud_provider"
    value = "GCP"
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

  values = [var.helm_values]
}