# Configuration resources for creating AWS EKS Cluster

data "aws_availability_zones" "available" {}

data "aws_eks_cluster" "eks" {
  name = module.eks.cluster_name
  depends_on = [
    module.eks
  ]

}

resource "terraform_data" "setup-kubeconfig" {
  provisioner "local-exec" {
    command = "aws eks update-kubeconfig --name ${var.cluster_name} --region ${var.region}"
  }
  depends_on = [
    module.eks
  ]
}

data "aws_eks_cluster_auth" "eks" {
  name = module.eks.cluster_name
  depends_on = [
    module.eks
  ]
}

# Set up an initial VPC for the instances to run in.
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "3.19.0"

  name = "grid-vpc"

  cidr = "10.0.0.0/16"
  azs  = slice(data.aws_availability_zones.available.names, 0, 3)

  private_subnets = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
  public_subnets  = ["10.0.4.0/24", "10.0.5.0/24", "10.0.6.0/24"]

  enable_nat_gateway   = true
  single_nat_gateway   = true
  enable_dns_hostnames = true

  public_subnet_tags = {
    "kubernetes.io/cluster/${var.cluster_name}" = "shared"
    "kubernetes.io/role/elb"                    = 1
  }

  private_subnet_tags = {
    "kubernetes.io/cluster/${var.cluster_name}" = "shared"
    "kubernetes.io/role/internal-elb"           = 1
  }
}

# Configure the actual cluster resource with your AWS account.
module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "19.10.1"

  cluster_name    = var.cluster_name
  cluster_version = var.cluster_version

  node_security_group_enable_recommended_rules = true

  vpc_id                         = module.vpc.vpc_id
  subnet_ids                     = module.vpc.private_subnets
  cluster_endpoint_public_access = true

  enable_irsa = true

  eks_managed_node_group_defaults = {
    ami_type = "AL2_x86_64"
    iam_role_additional_policies = {
      additional = aws_iam_policy.lb-policy.arn
    }

    tags = {
      "k8s.io/cluster-autoscaler/${var.cluster_name}" = "owned"
      "k8s.io/cluster-autoscaler/enabled" = true
    }
  }

  eks_managed_node_groups = {
    primary = {

      name = "node-group-1"

      instance_types = ["${var.instance_type}"]

      min_size     = var.node_count
      max_size     = var.node_count_max
      desired_size = var.node_count

    }
  }


}


data "aws_iam_policy_document" "lb_policies" {
  statement {
    actions = [ "ec2:DescribeVpcs",
          "ec2:DescribeSecurityGroups",
          "ec2:DescribeInstances",
          "elasticloadbalancing:DescribeTargetGroups",
          "elasticloadbalancing:DescribeTargetHealth",
          "elasticloadbalancing:ModifyTargetGroup",
          "elasticloadbalancing:ModifyTargetGroupAttriblutes",
          "elasticloadbalancing:RegisterTargets",
          "elasticloadbalancing:DeregisterTargets",
          "elasticloadbalancing:DescribeLoadBalancers" ]

    effect = "Allow"
    resources = [ "*" ]
  }
}

resource "aws_iam_policy" "lb-policy" {
  policy = data.aws_iam_policy_document.lb_policies.json
}


module "lb_role" {
  source    = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"

  role_name = "aws-load-balancer-controller"
  attach_load_balancer_controller_policy = true

  oidc_providers = {
    main = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:aws-load-balancer-controller"]
    }
  }

  depends_on = [
    module.eks
  ]
}



resource "kubernetes_secret" "service-account-secret" {
  provider = kubernetes.post-cluster
  metadata {
    name = "aws-load-balancer-controller"
  }
}

resource "kubernetes_service_account" "service-account" {
  provider = kubernetes.post-cluster

  secret {
    name = kubernetes_secret.service-account-secret.metadata[0].name
  }

  metadata {
    name      = "aws-load-balancer-controller"
    namespace = "kube-system"
    labels = {
      "app.kubernetes.io/name"      = "aws-load-balancer-controller"
      "app.kubernetes.io/component" = "controller"
    }
    annotations = {
      "eks.amazonaws.com/role-arn"               = module.lb_role.iam_role_arn
      "eks.amazonaws.com/sts-regional-endpoints" = "true"
    }
  }
  depends_on = [
    module.lb_role  
  ]
}


locals {
  autoscaler-sa-name = "aws-auto-scaler-sa"
  autoscaler-sa-namespace = "kube-system"
}

resource "aws_iam_policy" "autoscaler-policy" {
  name = "autoscaler-policy"
  policy = jsonencode(jsondecode(<<EOF
      {
        "Version": "2012-10-17",
        "Statement": [
            {
                "Sid": "VisualEditor0",
                "Effect": "Allow",
                "Action": [
                    "autoscaling:SetDesiredCapacity",
                    "autoscaling:TerminateInstanceInAutoScalingGroup"
                ],
                "Resource": "*",
                "Condition": {
                    "StringEquals": {
                        "aws:ResourceTag/k8s.io/cluster-autoscaler/${var.cluster_name}": "owned"
                    }
                }
            },
            {
                "Sid": "VisualEditor1",
                "Effect": "Allow",
                "Action": [
                    "autoscaling:DescribeAutoScalingInstances",
                    "autoscaling:DescribeAutoScalingGroups",
                    "ec2:DescribeLaunchTemplateVersions",
                    "autoscaling:DescribeTags",
                    "autoscaling:DescribeLaunchConfigurations",
                    "ec2:DescribeInstanceTypes"
                ],
                "Resource": "*"
            }
        ]
    }
    EOF
    ))
}

resource "aws_iam_role" "autoscaler-role" {
  name = "autoscaler-role"
  assume_role_policy = <<EOF
    {
      "Version": "2012-10-17",
      "Statement": [
        {
          "Effect": "Allow",
          "Principal": {
            "Federated": "${module.eks.oidc_provider_arn}"
          },
          "Action": "sts:AssumeRoleWithWebIdentity",
          "Condition": {
            "StringEquals": {
              "${module.eks.oidc_provider}:aud": "sts.amazonaws.com",
              "${module.eks.oidc_provider}:sub": "system:serviceaccount:${local.autoscaler-sa-namespace}:${local.autoscaler-sa-name}"
            }
          }
        }
      ]
    }
    EOF
    
  inline_policy {
    name = "autoscaler-role-policy"
    policy = aws_iam_policy.autoscaler-policy.policy
  }
}


resource "kubernetes_secret" "autoscaler-service-account-secret" {
  provider = kubernetes.post-cluster
  metadata {
    name = local.autoscaler-sa-name
  }
}

resource "kubernetes_service_account" "autoscaler-service-account" {
  provider = kubernetes.post-cluster

  secret {
    name = local.autoscaler-sa-name
  }

  metadata {
    name      = local.autoscaler-sa-name
    namespace = local.autoscaler-sa-namespace

    labels = {
      "app.kubernetes.io/name"      = local.autoscaler-sa-name
      "app.kubernetes.io/component" = "controller"
    }
    annotations = {
      "eks.amazonaws.com/role-arn"               = aws_iam_role.autoscaler-role.arn
      "eks.amazonaws.com/sts-regional-endpoints" = "true"
    }
  }
  depends_on = [
    module.eks,
    aws_iam_role.autoscaler-role
  ]
}


module "iam_eks_role" {
  source    = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  role_name = "autoscaler-eks-role"

  role_policy_arns = {
    policy = aws_iam_policy.autoscaler-policy.arn
  }

  oidc_providers = {
    one = {
      provider_arn = module.eks.oidc_provider_arn
      namespace_service_accounts = ["${local.autoscaler-sa-namespace}:${local.autoscaler-sa-name}"]
    }
  }
}