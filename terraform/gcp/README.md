
# Deploying

To deploy for GCP, you must:
1. Have the gcloud CLI installed (and initialized, with `gcloud init`), and be logged in (run `gcloud auth application-default login`)
2. Create a terraform.tfvars with the ID of the GCP project to deploy to:
    ```
    project_id = "projectid-123456"
    ```
3. Run `terraform apply`



After, you can run `gcloud container clusters get-credentials $(terraform output -raw cluster-name) --region $(terraform output -raw cluster-location)` to automatically configure kubectl