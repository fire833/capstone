
# Deploying

To deploy for Azure, you must:
1. Have the Azure CLI installed, and be logged in (run `az login`)
2. Create an active directory service principal account by running `az ad sp create-for-rbac --skip-assignment`
3. Create a terraform.tfvars with the appId and password which that command outputs:
    ```
    appId = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
    password = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
    ```
4. Run `terraform apply`

After, you can run `az aks get-credentials --resource-group $(terraform output -raw resource_group_name) --name $(terraform output -raw kubernetes_cluster_name)` to automatically configure kubectl