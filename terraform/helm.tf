variable gke {}

provider "helm" {
  kubernetes {
    host = gke.name
  }
}

resource "helm_release" "selenium-test-runner" {
  name = "test-runner"

  # Ask Aaron about link
  repository = ""
  chart = ""

  values = [
    "${path.root}"
  ]
}