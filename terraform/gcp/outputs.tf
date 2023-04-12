output "grid-endpoint" {
  value = google_compute_global_address.default.address
}

output "cluster-name" {
  value = google_container_cluster.primary.name
}

output "cluster-location" {
  value = google_container_cluster.primary.location
}