

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