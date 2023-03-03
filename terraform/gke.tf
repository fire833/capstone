variable "gke_username" {
  default = ""
  description = "gke_username"
}

variable "gke_password" {
  default = ""
  descript = "gke_password"
}

resource "google_container_cluster" "gke" {
  # Define in different TF file ?
  name = ""
  location = ""

  # Edit later
  initial_node_count = 1

  # Define in different TF file
  network = 
  subnetwork = 
}

# May need to use one of the helm charts for this
resource "google_container_node_pool" "gke_nodes" {

  name = 
  location = 
  cluster = google_container_cluster.gke
  node_count = 

  node_config {
    oauth_scopes = []

    labels = {}

    machine_type = 
    tags = 
  }
  
}