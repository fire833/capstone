variable "gke_region" {
  default = "us-central1"
  description = "GKE region"
}

variable "gke_projectid" {
  description = "GKE project ID"
}

resource "google_service_account" "service_account" {
  account_id = "${google_container_cluster.name}-node-service-acount"
  display_name = "Node Service Account"
}

resource "google_container_cluster" "gke" {
  
  name = "${var.gke_projectid}-gke"
  project = var.gke_projectid
  location = var.gke_region

  # Node pool is managed by autoscaler
  remove_default_node_pool = true
  initial_node_count = 1

  # Defined in 'gke_vpc.tf'
  # network = "${var.gke_projectid}-vpc"
}

# May need to use one of the helm charts for this
resource "google_container_node_pool" "gke_nodes" {

  name = "${var.gke_projectid}-nodes"
  project = var.gke_projectid
  location = google_container_cluster.gke.location

  cluster = google_container_cluster.gke.name
  node_count = 1

  node_config {
    preemptible = true
    machine_type = "n1-standard-1"
    
    service_account = google_service_account.service_account.email
    oauth_scopes = [
      "https://www.googleapis.com/auth/cloud-platform"
    ]
  }
}