

## Tearing down

You need to manually delete the load balancer which gets created to make the grid externally accessible.
From the console in the region with the cluster, go to EC2 -> load balancers, and delete the load balancer,
then run `terraform destroy`. 

The load balancer will be cleaned up at some point once it is disassociated from the 
hub service, but it will prevent the VPC from being deleted until that time comes,
so it's best to manually delete it, otherwise the destroy command will time out.

## Common Issues


### Dial Tcp: No such host
Sometimes, an apply will break mid-run with the error 
`Get "<URL>: dial tcp: lookup https://***.eks.amazonaws.com on <IP> no such host`.

To fix this, you need to:
1. `aws eks update-kubeconfig --name grid_cluster --region <cluster region>`
2. `export KUBE_CONFIG_PATH=~/.kube/config`, where ~/.kube/config is wherever update-kubeconfig has updated (~/.kube/config is the default)


### Can't view cluster in the console

This is likely because you are logged onto the console using a different IAM user
than the one which your AWS CLI is configured with.

You are likely viewing the cluster from the root IAM user, and you created
the cluster using a different IAM user.
You can either log in to the console as that IAM user, or follow these steps to add
the root user to the configmap:

1. Run `kubectl edit configmap aws-auth -n kube-system` to open the editor
    - You may need to `export KUBE_EDITOR=vim` or `export KUBE_EDITOR=nano` if you don't have `vi` installed
2. Add the following to the config map, at the same level as mapRoles
    ```
    mapUsers: |
    - userarn: arn:aws:iam::[account_id]:root
        groups:
        - system:masters
    ```