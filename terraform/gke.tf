# variable "gke_username" {
#   default = ""
#   description = "gke_username"
# }

# variable "gke_password" {
#   default = ""
#   description = "gke_password"
# }

resource "google_container_cluster" "gke" {
  
  name = "${var.gke_projectid}-gke"
  location = var.gke_region

  # Node pool is managed by autoscaler
  remove_default_node_pool = true
  initial_node_count = 1

  # Defined in 'gke_vpc.tf'
  network = "${var.gke_projectid}-vpc"
}

# May need to use one of the helm charts for this
resource "google_container_node_pool" "gke_nodes" {

  name = "${var.gke_projectid}-nodes"
  location = google_container_cluster.gke.location
  
  cluster = google_container_cluster.gke.name
  node_count = 1

  node_config {
    # oauth_scopes = []
    labels = {
      env = var.gke_projectid
    }

    machine_type = "n1-standard-1"
    tags = ["gke-node", "${google_container_cluster.gke.name}"]
  }
}