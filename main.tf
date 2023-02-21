provider "kubernetes" {
  config_path = ""
  config_context = "minikube"
}

resource "kubernetes_namespace" "test" {
  metadata {
    name = "k8s"
  }
}

resource "kubernetes_deployment" "test" {
    metadata {
        name = "terraform"
        namespace = "k8s"
    }

    spec {
        replicas = 1

    }
}