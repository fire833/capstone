

## Common Issues


### Dial Tcp: No such host
Sometimes, an apply will break mid-run with the error 
`Get "<URL>: dial tcp: lookup https://***.eks.amazonaws.com on <IP> no such host`.

To fix this, you need to:
1. `aws eks update-kubeconfig --region <cluster region> --name grid_cluster`
2. `export KUBE_CONFIG_PATH=~/.kube/config`, where ~/.kube/config is wherever update-kubeconfig has updated (~/.kube/config is the default)


### Can't view cluster in the console
In order to view the cluster in the console, you have to add your account to the aws auth configmap.
To do this, follow this [stackoverflow answer](https://stackoverflow.com/a/70980613), which amounts to:

1. Run `kubectl edit configmap aws-auth -n kube-system` to open the editor
    - You may need to `export KUBE_EDITOR=vim` or `export KUBE_EDITOR=nano` if you don't have `vi` installed
2. Add the following to the config map, at the same level as mapRoles
    ```
    mapUsers: |
    - userarn: arn:aws:iam::[account_id]:root
        groups:
        - system:masters
    ```