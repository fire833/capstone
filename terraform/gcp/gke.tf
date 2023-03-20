# Configuration resources for creating GCP GKE Cluster.


# Retrieve an access token from the gcloud CLI
data "google_client_config" "default" {}

# VPC creation
resource "google_compute_network" "vpc" {
  name                    = "grid-cluster-vpc"
  auto_create_subnetworks = "false"
}

# Subnet
resource "google_compute_subnetwork" "subnet" {
  name          = "grid-cluster-subnet"
  region        = var.region
  network       = google_compute_network.vpc.name
  ip_cidr_range = "10.10.0.0/24"
}

# Configure public IP for ingress.
resource "google_compute_global_address" "default" {
  project      = var.project_id
  name         = "gcp-hub-global-ip"
  address_type = "EXTERNAL"
  ip_version   = "IPV4"
}

# Configure the actual cluster resource with your GCP account.
resource "google_container_cluster" "primary" {
  name     = var.cluster_name
  location = var.region

  # Set a minimum version to the cluster.
  min_master_version = var.cluster_version

  # We can't create a cluster with no node pool defined, but we want to only use
  # separately managed node pools. So we create the smallest possible default
  # node pool and immediately delete it.
  initial_node_count = var.node_count

  master_auth { 
    client_certificate_config {
      issue_client_certificate = false
    }
  }

  cluster_autoscaling {
    enabled = true

    resource_limits {
      maximum = local.cluster_autoscaling_max_cpu_cores
      resource_type = "cpu"
    }
  
    resource_limits {
      maximum = local.cluster_autoscaling_max_gb_ram
      resource_type = "memory"
    }
  }

  node_config {
    oauth_scopes = [
      "https://www.googleapis.com/auth/logging.write",
      "https://www.googleapis.com/auth/monitoring",
    ]

    labels = {
      env = var.project_id
    }

    # preemptible  = true
    machine_type = "e2-medium"
    tags         = ["gke-node", "${var.project_id}-gke"]
    metadata = {
      disable-legacy-endpoints = "true"
    }
  }

  network    = google_compute_network.vpc.name
  subnetwork = google_compute_subnetwork.subnet.name
}
