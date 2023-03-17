
resource "helm_release" "grid_cluster" {
  name       = "grid-cluster-chart"
  repository = "https://fire833.github.io/capstone/"
  chart      = "selenium-grid-cluster"

  set {
    name  = "cloud_provider"
    value = "AWS"
  }
}

# resource "helm_release" "nginx_ingress" {
#   name       = "aws-load-balancer"
#   repository = "https://aws.github.io/eks-charts/"
#   chart      = "aws-load-balancer-controller"

#   namespace = "kube-system"

#   set {
#     name = "clusterName"
#     value = module.eks.cluster_name
#   }

#   # set {
#   #   name = "serviceAccount.create"
#   #   value = true
#   # }
# }
