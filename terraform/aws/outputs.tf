
data "aws_lb" "hub_svc_lb" {
  tags = {
    "kubernetes.io/cluster/${var.cluster_name}" = "owned"
    "kubernetes.io/service-name" = "${var.deploy_namespace}/hubsvc"
  }
  depends_on = [
    helm_release.grid_cluster,
    helm_release.nginx_ingress
  ]
}

output "grid-endpoint" {
  value = data.aws_lb.hub_svc_lb.dns_name
  depends_on = [
    data.aws_lb.hub_svc_lb
  ]
}
