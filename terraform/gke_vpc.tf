# provider "google" {
#     project = var.gke_projectid
#     region = var.gke_region
# }

# # VPC
# resource "google_compute_network" "vpc" {
#     name = "${var.gke_projectid}-vpc"
#     auto_create_subnetworks = false
# }

# # Subnet, if needed (likely connect exclusively to hub router).
# resource "google_compute_subnetwork" "subnet" {
#     name = "${google_compute_network.vpc.name}-subnet"
#     region = var.gke_region
#     network = google_compute_network.vpc
#     ip_cidr_range = ""
# }