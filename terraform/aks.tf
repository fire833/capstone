# AKS Cluster
resource "azurerm_kubernetes_cluster" "aks" {
    name = 
    location = 
    resource_group_name = 
    dns_prefix = 

    default_node_pool {
      name = ""
      node_count = 1
      vm_size = ""
      os_disk_size_gb = 15
    }

    service_principal {
      client_id = var.aksId
      client_secret = var.aksPswd
    }

    # role_role_based_access_control {
    #     enabled = False
    # }

    # tags {

    # }
}