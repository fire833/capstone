
resource "helm_release" "grid_cluster" {
  name       = "grid-cluster-chart"
  repository = "https://fire833.github.io/capstone/"
  chart      = "selenium-grid-cluster"

  namespace = var.deploy_namespace

  set {
    name  = "cloud_provider"
    value = "AWS"
  }

  depends_on = [
    helm_release.nginx_ingress
  ]
}

resource "helm_release" "nginx_ingress" {
  name       = "aws-load-balancer"
  repository = "https://aws.github.io/eks-charts/"
  chart      = "aws-load-balancer-controller"

  namespace = "kube-system"

  set {
    name = "clusterName"
    value = module.eks.cluster_name
  }

  set {
    name = "serviceAccount.create"
    value = "false"
  }

  set {
    name = "serviceAccount.name"
    value = "aws-load-balancer-controller"
  }

  set {
    name = "vpcId"
    value = module.vpc.vpc_id
  }

  set {
    name = "region"
    value = var.region
  }

  depends_on = [
    kubernetes_service_account.service-account
  ]
}
