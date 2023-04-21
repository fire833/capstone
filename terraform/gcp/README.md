
# Deploying

To deploy for GCP, you must:
1. Have the gcloud CLI installed (and initialized, with `gcloud init`), and be logged in (run `gcloud auth application-default login`)
2. Create a terraform.tfvars with the ID of the GCP project to deploy to:
    ```
    project_id = "projectid-123456"
    ```
3. Run `terraform apply`



After, you can run `gcloud container clusters get-credentials $(terraform output -raw cluster-name) --region $(terraform output -raw cluster-location)` to automatically configure kubectl


# Autoscaling configuration

GCP's cluster autoscaling solution scales both horizontally and vertically.
Because of this, you do not specify max/min node counts for scaling, you choose
the max/min number of CPU cores and RAM.
This will be inferred by your set maximum replicas for each browser,
times the CPU and RAM limits set for selenium nodes.

# Common Issues and Maintenance

To resolve Kubernetes connectivity issues (similar to: `Kubernetes cluster unreachable: invalid configuration: no configuration has been provided, try setting KUBERNETES_MASTER environment variable`), do the following:
1. Run `gcloud container clusters get-credentials $(terraform output -raw cluster-name) --region $(terraform output -raw cluster-location)`
2. Run `export KUBE_CONFIG_PATH=~/.kube/config`