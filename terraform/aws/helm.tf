
resource "helm_release" "grid_cluster" {
  name       = "grid-cluster-chart"
  repository = "https://fire833.github.io/capstone/"
  chart      = "selenium-grid-cluster"

  set {
    name  = "cloud_provider"
    value = "AWS"
  }
}
