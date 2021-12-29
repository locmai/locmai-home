---
title: 'How to use HashiCorp Vault and ArgoCD for GitOps'
description: "How to Use HashiCorp Vault and ArgoCD for GitOps: ArgoCD + Vault + vault-argocd-plugin"
date: 2021-09-27
author:
  name: Loc Mai
tags:
  - gitops 
  - kubernetes
  - vault
published: true
layout: layouts/post.njk
---

## Context

There is various way to manage your secrets, HashiCorp Vault just happened to be a pretty much widely known approach that I had in mind. So for Humble project, I used it as the core secret management system that go along with Kubernetes secrets themselves. So in this post I'd like to share how I use HashipCorp Vault with ArgoCD to deploy secrets across my system.

For my own convenience: Vault = HashiCorp Vault

## Injecting Vault secrets into Kubernetes pods via sidecar

I followed through the official [article](https://www.hashicorp.com/blog/injecting-vault-secrets-into-kubernetes-pods-via-a-sidecar). The idea is enable Kubernetes authentication in Vault, bind a Kubernetes Service Account to a role, then setting that role to allow the pods that go with the Service Account to read the secrets in a scoped manner.

Started installing it with the official Helm chart for Vault via Terraform: [init-resources.tf](https://github.com/locmai/humble/blob/7e5eaf271b5b88f83ea8460935d195f39bb11acd/infras/terraform/init-resources.tf#L16) 

The dependency here will be longhorn for the persitent block storage that will be Vault backend:

``` hcl/
depends_on       = [helm_release.longhorn]
```

Remember to switch the injector support on in the values.yaml:

``` yaml/
injector:
  enabled: true
```

Vault will be up and running in a few seconds, we will initialize and unseal it first, step by step:

``` bash/
kubectl exec -n vault -ti vault-0 /bin/sh
vault operator init
vault operator unseal
```

Now we define the app policy, which better be provisioned via Terraform as well:

``` yaml/
resource "vault_policy" "postgresql_read_only" {
  depends_on = [helm_release.vault]
  name       = "postgresql_read_only"
  policy     = <<EOT
  path "secret/postgresql/*" {
    capabilities = ["read"]
}
EOT
}
```

First, enable the Kubernets authentication

``` bash/
vault auth enable kubernetes
```

Now if you were on one of the control plane node of your cluster, the quickest way to set this up is:

``` bash/
vault write auth/kubernetes/config \
   token_reviewer_jwt="$(cat /var/run/secrets/kubernetes.io/serviceaccount/token)" \
   kubernetes_host=https://${KUBERNETES_PORT_443_TCP_ADDR}:443 \
   kubernetes_ca_cert=@/var/run/secrets/kubernetes.io/serviceaccount/ca.crt

vault write auth/kubernetes/role/myapp \
   bound_service_account_names=app \
   bound_service_account_namespaces=demo \
   policies=app \
   ttl=1h
```

Otherwise, you will have to get the JWT token and the ca.crt from one of your Service Account:

``` yaml/
export VAULT_SA_NAME=$(kubectl get sa vault-auth) \
    --output jsonpath="{{- '{' -}}.secrets[*]['name']{{- '}' -}}")
export SA_JWT_TOKEN=$(kubectl get secret $VAULT_SA_NAME \
    --output 'go-template={{- '{{' -}} .data.token {{- '}}' -}}' | base64 --decode)
export SA_CA_CRT=$(kubectl config view --raw --minify --flatten \
    --output 'jsonpath={{- '{' -}}.clusters[].cluster.certificate-authority-data{{- '}' -}}' | base64 --decode)
export K8S_HOST=$(kubectl config view --raw --minify --flatten \
    --output 'jsonpath={{- '{' -}}.clusters[].cluster.server{{- '}' -}}')

vault write auth/kubernetes/config \
        token_reviewer_jwt="$SA_JWT_TOKEN" \
        kubernetes_host="$K8S_HOST" \
        kubernetes_ca_cert="$SA_CA_CRT"
```

Let's create a role and bind that role to our Service Account:

``` yaml/
vault write auth/kubernetes/role/postgresql_read_only \
   bound_service_account_names=apps \
   bound_service_account_namespaces=apps \
   policies=postgresql_read_only \
   ttl=1h
```

Now let's put some secrets in there!

``` yaml/
vault kv put secret/postgresql/data \
      username='databaseuser' \
      password='suP3rsec(et!' \
      ttl='30s'
```

Now we can inject it in our pods, refer to the document here [https://www.vaultproject.io/docs/platform/k8s/injector](https://www.vaultproject.io/docs/platform/k8s/injector).

As an example, in my ArgoCD apps folder, I injected the secrets via the ENV VAR "DSN" for Ory Kratos to take it as the connection string to the PostgreSQL database: [ory-kratos.yaml](https://github.com/locmai/humble/blob/7e5eaf271b5b88f83ea8460935d195f39bb11acd/apps/argocd/templates/ory-kratos.yaml#L80-L87)

``` yaml/
annotations:
  vault.hashicorp.com/agent-inject: "true"
  vault.hashicorp.com/agent-inject-secret-config: 'secret/postgresql/data'
  vault.hashicorp.com/role: "postgresql_read_only"
  vault.hashicorp.com/agent-inject-template-config: |
    {{'{{`{{ with secret "secret/postgresql/data" -}}'}}
      export DSN="postgres://{{- '{{' -}} .Data.data.kratos_username {{- '}}' -}}:{{- '{{' -}} .Data.data.kratos_password {{- '}}' -}}@postgresql:5432/{{- '{{' -}} .Data.data.kratos_database {{- '}}' -}}?sslmode=disable&max_conns=20&max_idle_conns=4"
    {{'{{- end }}`}}'}}
```

And let the sidecar do it's job!


## Using vault-argocd-plugin

Injecting the secrets into pods is quite easy and straightforward. But what if we wanted to inject it on the fly to other resources like Kubernetes secrets or custom resources? And this is when vault-argocd-plugin comes to aid us. 

[vault-argocd-plugin](https://github.com/IBM/argocd-vault-plugin) is an ArgoCD plugin provided by IBM for helping us doing so. To set this up, first install the executable binary via the initContainer, I used the Helm chart which includes the YAML code for that:

``` yaml/
repoServer:
    metrics: 
        enabled: true
        serviceMonitor:
            enabled: true
    image:
        tag: v2.0.0

    volumes:
    - name: custom-tools
      emptyDir: {}

    initContainers:
    - name: download-tools
      image: alpine:3.8
      command: [sh, -c]
      args:
        - >-
          wget -O argocd-vault-plugin
          https://github.com/IBM/argocd-vault-plugin/releases/download/v1.1.1/argocd-vault-plugin_1.1.1_linux_amd64 &&
          chmod +x argocd-vault-plugin &&
          mv argocd-vault-plugin /custom-tools/
      volumeMounts:
        - mountPath: /custom-tools
          name: custom-tools
    volumeMounts:
    - name: custom-tools
      mountPath: /usr/local/bin/argocd-vault-plugin
      subPath: argocd-vault-plugin
```

Next, we configure plugin so ArgoCD would be able to know how to use it:

``` yaml/
server:
  config:
    configManagementPlugins: |-
      - name: argocd-vault-plugin
        generate:
          command: ["argocd-vault-plugin"]
          args: ["generate", "./"]
      - name: argocd-vault-plugin-helm
        init:
          command: [sh, -c]
          args: ["helm dependency build"]
        generate:
          command: ["sh", "-c"]
          args: ["helm template $ARGOCD_APP_NAME . | argocd-vault-plugin generate -"]
```


Here is the values.yaml file: [argocd.yaml](https://github.com/locmai/humble/blob/7e5eaf271b5b88f83ea8460935d195f39bb11acd/infras/terraform/helm-values/argocd.yaml)

Let's put a new secret

``` yaml/
vault kv put secret/humble/demo \
  username='locmai' \
  ttl='30s'
```

Now, create a policy and a role:

``` yaml/
kubectl exec -ti vault-0 /bin/sh

cat <<EOF > /home/vault/humble-policy.hcl
path "secret/humble/*" {
  capabilities = ["read"]
}
EOF

vault policy write humble /home/vault/humble-policy.hcl

vault write auth/kubernetes/role/myhumbledemo \
   bound_service_account_names=default \
   bound_service_account_namespaces=argocd \
   policies= humble\
   ttl=1h
```

Now we could put it all together nice and easy with ArgoCD, let's define a Kubernetes secret manifest that we would like to inject the secret value into:

``` yaml/
# demo/secrets.yaml
kind: Secret
apiVersion: v1
metadata:
  name: humble-example-secret
  namespace: argocd
  annotations:
    avp.kubernetes.io/path: "secret/humble/data/demo"
type: Opaque
stringData:
  username: <username>
```

The above spec define that we would want to read the secret/humble/demo path (`/data/` is the new path pattern for kv-v2 engine) and the secret with the key `username` will be injected in to <username> placeholder.

Now from the ArgoCD application, we could simply define it to use the plugin as:

``` yaml/
spec:
  source:
    path: 'demo'
    repoURL: git@github.com:locmai/humble.git
    targetRevision: main
    plugin:
      name: argocd-vault-plugin
      env:
        - name: VAULT_ADDR
          value: http://vault-ui.vault.svc.cluster.local:8200
        - name: AVP_TYPE
          value: vault
        - name: AVP_AUTH_TYPE
          value: k8s
        - name: AVP_K8S_ROLE
          value: argocd-server
```

And the secret injected nice and easy!


<img alt="Secret injected" src="https://raw.githubusercontent.com/locmai/locmai-home/master/img/argocdsecrets.png" width="40%" height="40%">

Here's some other options people are doing GitOps secrets: [https://argoproj.github.io/argo-cd/operator-manual/secret-management](https://argoproj.github.io/argo-cd/operator-manual/secret-management/)

And that's it! There is other ways as well, will post about them when I had the time to try.
