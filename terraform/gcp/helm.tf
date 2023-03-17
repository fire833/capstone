resource "helm_release" "example" {
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
}