
data "aws_lb" "hub_svc_lb" {
  tags = {
    "elbv2.k8s.aws/cluster" = "${var.cluster_name}"
    "service.k8s.aws/stack" = "${var.deploy_namespace}/hubsvc"
  }

}

output "grid-endpoint" {
  value = "http://${data.aws_lb.hub_svc_lb.dns_name}:9994/"

}
