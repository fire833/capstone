# VPC
resource "google_compute_network" "vpc" {
    name = "${var.gke_projectid}-vpc"
    auto_create_subnetworks = false
}

# # Subnet, if needed
# resource "google_compute_subnetwork" "subnet" {
#     name = 
#     region = 
#     network = google_compute_network.vpc.name
#     ip_cidr_range = ""
# }