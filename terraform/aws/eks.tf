# Configuration resources for creating AWS EKS Cluster

data "aws_availability_zones" "available" {}

data "aws_eks_cluster" "eks" {
  name = module.eks.cluster_name
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
  version = "19.5.1"

  cluster_name    = var.cluster_name
  cluster_version = var.cluster_version

  vpc_id                         = module.vpc.vpc_id
  subnet_ids                     = module.vpc.private_subnets
  cluster_endpoint_public_access = true

  aws_auth_node_iam_role_arns_non_windows = [""]

  eks_managed_node_group_defaults = {
    ami_type = "AL2_x86_64"

  }

  eks_managed_node_groups = {
    primary = {
      name = "node-group-1"

      instance_types = ["t3.small"]

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
    # principals {
    #   type = "Service"
    #   identifiers = [ "ec2.amazonaws.com" ]
    # }
  }
}

data "aws_iam_policy_document" "assume_role_policy" {
  statement {
    actions = [ "sts:AssumeRole" ]
    effect = "Allow"
    principals {
      type = "Service"
      identifiers = [ "ec2.amazonaws.com" ]
    }
  }
}

resource "aws_iam_role" "lb_role" {
  name = "ab_role"

  assume_role_policy = data.aws_iam_policy_document.assume_role_policy.json

  inline_policy {
    policy = data.aws_iam_policy_document.lb_policies.json
  }

  # assume_role_policy = <<EOT
  # {
  #   "Statement" : [
  #     {
  #       "Action" : [
  #         "ec2:DescribeVpcs",
  #         "ec2:DescribeSecurityGroups",
  #         "ec2:DescribeInstances",
  #         "elasticloadbalancing:DescribeTargetGroups",
  #         "elasticloadbalancing:DescribeTargetHealth",
  #         "elasticloadbalancing:ModifyTargetGroup",
  #         "elasticloadbalancing:ModifyTargetGroupAttributes",
  #         "elasticloadbalancing:RegisterTargets",
  #         "elasticloadbalancing:DeregisterTargets"
  #       ],
  #       "Effect" : "Allow",
  #       "Resource" : "*"
  #     }
  #   ],
  #   "Version" : "2012-10-17"
  # }
  # EOT
}
