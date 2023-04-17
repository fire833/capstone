
resource "helm_release" "grid_cluster" {
  name       = "grid-cluster-chart"
  repository = "https://fire833.github.io/capstone/"
  chart      = "selenium-grid-cluster"

  namespace = var.deploy_namespace

  timeout = 600
  
  set {
    name  = "cloud_provider"
    value = "AWS"
  }

  set {
    name = "prometheus-adapter.hostNetwork.enabled"
    value = true
  }

  set {
    name = "prometheus-adapter.dnsPolicy"
    value = "ClusterFirstWithHostNet"
  }

  set {
    name = "prometheus-adapter.logLevel"
    value = 100
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

  depends_on = [
    helm_release.load_balancer_controller
  ]

  values = [var.helm_values]
}

resource "helm_release" "load_balancer_controller" {
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
    module.eks,
    kubernetes_service_account.service-account
  ]
}


resource "helm_release" "cluster_autoscaler" {
  name = "aws-cluster-autoscaler"
  repository = "https://kubernetes.github.io/autoscaler/"
  chart = "cluster-autoscaler"

  namespace = "kube-system"

  set {
    name = "autoDiscovery.clusterName"
    value = var.cluster_name
  }

  set {
    name = "awsRegion"
    value = var.region
  }

  set {
    name = "cloudProvider"
    value = "aws"
  }

  set {
    name = "rbac.serviceAccount.create"
    value = false
  }

  set {
    name = "rbac.serviceAccount.name"
    value = kubernetes_service_account.autoscaler-service-account.metadata[0].name
  }

  depends_on = [
    module.eks,
    kubernetes_service_account.autoscaler-service-account
  ]
}

resource "helm_release" "metrics_server_release" {
  name = "metrics-server-release"
  repository = "https://kubernetes-sigs.github.io/metrics-server/"
  chart = "metrics-server"

  namespace = "kube-system"

  depends_on = [
    module.eks
  ]
}