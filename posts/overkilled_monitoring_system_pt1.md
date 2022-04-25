---
title: 'Overkilled homelab monitoring system, Pt. 1'
description: "How-to guide to deploy a high availability Cortex system on bare-metal"
date: 2022-04-25
author:
  name: Loc Mai
tags:
  - cortex
  - kubernetes
  - monitoring
layout: layouts/post.njk
---

We switched from DataDog to a new self-managed system developed with Cortex, Grafana, OpenTelemetry, and other tools last year. I was planning to write a blog post on the entire process, but realized there would be a lot to cover. So I've decided to start with the most important one, which is also the easiest to approach and apply.

Let's begin by addressing the topic of "what is Cortex." On the first page of [Cortex](cortexmetrics.io), it gives the clearest definition I could think of for Cortex:

> Cortex provides horizontally scalable, highly available, multi-tenant, long term storage for Prometheus.

It's designed with a microservices architecture in mind, with a focus on scalability and high availability.

![Architecture](https://cortexmetrics.io/images/architecture.png)

It appears jumbled, but we only need to worry about two key metrics paths: write (in) and read (out).

To go in the system, the metrics will go through Nginx instances, [distributors](https://cortexmetrics.io/docs/architecture/#distributor), and [ingesters](https://cortexmetrics.io/docs/architecture/#ingester). Then finally be written into the block storage. The instances of each service will communicate through a hash ring stored in key-value store like Consul or etcd to achieve the consistent hashing for the series shards and replications across the instance.

On the way out, the metrics will go through the [querier](https://cortexmetrics.io/docs/architecture/#querier), [query frontend](https://cortexmetrics.io/docs/architecture/#query-frontend), and Nginx as well. These services will handle the queries in PromQL query format, probably caching the results/indexes for reusing them in subsequent queries.

Other components, which will be detailed later in this blog, give various opt-in functionality.

> I'm going to write this up using simple Helm charts and commands to make it easier to implement. The [Humble](https://github.com/locmai/humble) project would be a better fit as a reference for the GitOps implementation.

## Distributed block storage setup

I run a bare-metal Kubernetes cluster, and provisioning persistent storage is one of the challenging aspects of the setup.

Fortunately, we have Longhorn! It provides a perfect highly available setup, incremental snaps and backups mechanism and cross-cluster disaster recovery.

Let's set it up real quick with Helm by adding the chart:

```
helm repo add longhorn https://charts.longhorn.io
helm repo update
```

Create a custom values file:

```yaml
# longhorn_values.yaml
persistence:
  defaultClassReplicaCount: 2
ingress:
  enabled: false
```
Then we create the namespace for it and install the chart:

```
helm install longhorn longhorn/longhorn --create-namespace -n longhorn-system -f longhorn_values.yaml
```

After the installation finished, check the default storage class we have applied:

```
kubectl get storageclasses.storage.k8s.io
NAME                 PROVISIONER          RECLAIMPOLICY   VOLUMEBINDINGMODE   ALLOWVOLUMEEXPANSION   AGE
longhorn (default)   driver.longhorn.io   Delete          Immediate           true                   3d19h
```

Taking a quick look into that one to see all the default values:

```
kubectl describe storageclasses.storage.k8s.io longhorn
Name:            longhorn
IsDefaultClass:  Yes
Annotations:     longhorn.io/last-applied-configmap=kind: StorageClass
apiVersion: storage.k8s.io/v1
metadata:
  name: longhorn
  annotations:
    storageclass.kubernetes.io/is-default-class: "true"
provisioner: driver.longhorn.io
allowVolumeExpansion: true
reclaimPolicy: "Delete"
volumeBindingMode: Immediate
parameters:
  numberOfReplicas: "2"
  staleReplicaTimeout: "30"
  fromBackup: ""
  fsType: "ext4"
,storageclass.kubernetes.io/is-default-class=true
Provisioner:           driver.longhorn.io
Parameters:            fromBackup=,fsType=ext4,numberOfReplicas=2,staleReplicaTimeout=30
AllowVolumeExpansion:  True
MountOptions:          <none>
ReclaimPolicy:         Delete
VolumeBindingMode:     Immediate
Events:                <none>
```

We have the `numberOfReplicas` to 2, as to me, it's okay to have 1 server being down and rely on the other one. You could choose 3 as the default one if cost saving wasn't the bigger concern than high availability.

For now, I don't have any backup pipeline/setting for the storage.

## Components deployment

### Consul

Add the Consul Helm chart:

```
helm repo add hashicorp https://helm.releases.hashicorp.com
helm repo update
```

Create a custom values file:

```yaml
# consul_values.yaml
server:
  replicas: 3
  affinity: ""
  topologySpreadConstraints: |
    - maxSkew: 1
      topologyKey: topology.kubernetes.io/zone
      whenUnsatisfiable: ScheduleAnyway
      labelSelector:
        matchLabels:
          app: consul
          release: consul
          component: server
ingress:
  enabled: false
```

I have switched from `affinity` spec to `topologySpreadConstraints` in order to control how the Consul pods are spread across the cluster.

Then we install the Helm chart:

```
helm install consul hashicorp/consul --set global.name=consul --create-namespace -n consul -f consul_values.yaml
```

### Write/Read components

Just like the other ones, we will pull the Cortex chart:

```
helm repo add cortex-helm https://cortexproject.github.io/cortex-helm-chart
helm repo update
```

And have our custom values file:

```yaml
# cortex_values.yaml
# PLACEHOLDER
```

Now that we've got everything in place, let's get started. To have the best understanding of the full configuration, I will explain separately the parts of it.

Set the `alertmanager` and `ruler` to turn off:

```yaml
# cortex_values.yaml
alertmanager:
  enabled: false
ruler:
  enabled: false
#...
```

We don't need these yet.

Setting for Nginx:

```yaml
# cortex_values.yaml
# ...
nginx:
  enabled: true
  replicas: 2
# ...
```

My cluster is relatively small, so 2 replicas are good enough to handle the traffic. 3 would be a great number for HA.

```yaml
# cortex_values.yaml
# ...
ingester:
  autoscaling:
    enabled: true
    minReplicas: 2
    maxReplicas: 5
  persistentVolume:
    enabled: true

  serviceMonitor:
    enabled: true

distributor:
  autoscaling:
    enabled: true
    minReplicas: 2
    maxReplicas: 5
  persistentVolume:
    enabled: true

  serviceMonitor:
    enabled: true

compactor:
  enabled: true
  replicas: 1

  serviceMonitor:
    enabled: true

  persistentVolume:
    enabled: true
    size: 2Gi
    storageClass: longhorn

table_manager:
  replicas: 1
# ...
```

For ingesters and distributors, I will have the autoscaling on the default one `targetCPUUtilizationPercentage: 80` from 2 to 5 instances. The `persistentVolume` will pick up the default storage class `longhorn` and start provisioning the persistent volumes on their own. This would be able to handle the traffic from my cluster and able to scale if any burst traffic added.

If you had more than one storage class and longhorn wasn't the default one, then you could set - for example the compactor - `compactor.persistentVolume.storageClass: longhorn`

The read components would be set to:
```yaml
# cortex_values.yaml
# ...
querier:
  replicas: 2

  serviceMonitor:
    enabled: true

query_frontend:
  replicas: 2

  serviceMonitor:
    enabled: true

store_gateway:
  replicas: 3

  serviceMonitor:
    enabled: true

  persistentVolume:
    enabled: true
    size: 2Gi
    storageClass: longhorn
# ...
```

I believe 2 is quite okay for my read traffic. I could add the HPA settings as well for these workloads later. 

A bunch of memcached instances will be provisioned as well:
```yaml
# cortex_values.yaml
# ...
memcached:
  enabled: false

# -- index read caching for legacy chunk storage engine
memcached-index-read:
  enabled: false
# -- index write caching for legacy chunk storage engine
memcached-index-write:
  enabled: false

memcached-frontend:
  enabled: true
  architecture: "high-availability"
  replicaCount: 2
  resources: {}

memcached-blocks-index:
  architecture: "high-availability"
  replicaCount: 2

memcached-blocks:
  architecture: "high-availability"
  replicaCount: 2

memcached-blocks-metadata:
  architecture: "high-availability"
  replicaCount: 2
# ...
```

I have disabled the legacy ones and enabled the latest ones for my usage.

For general configuration, we have a `config` spec to put some common settings in there.

```yaml
# cortex_values.yaml
# ...
config:
  storage:
    engine: blocks
    index_queries_cache_config:
      memcached:
        # -- How long keys stay in the memcache
        expiration: 1h
      memcached_client:
        # -- Maximum time to wait before giving up on memcached requests.
        timeout: 1s
  blocks_storage:
    backend: filesystem
    tsdb:
      dir: /data/tsdb
    bucket_store:
      sync_dir: /data/tsdb-sync
      bucket_index:
        enabled: true
# ...
```

Ingester K/V store and ingesting limit settings:

```yaml
# cortex_values.yaml
# ...
config:
  # ...
  limits:
    ingestion_rate: 150000
    max_series_per_user: 0
    max_series_per_metric: 0
  ingester:
    lifecycler:
      ring:
        kvstore:
          store: "consul"
          consul:
            host: consul-consul-server.apps:8500
# ...
```

I find 150000 samples per second is a reasonable number. The ingester K/V store setting hooks ingesters with the Consul we deployed before.

For accepting the samples from HA pairs of Prometheus - say HA setup - we have the following:

```yaml
# cortex_values.yaml
# ...
config:
  # ...
  limits:
    accept_ha_samples: true
    ha_cluster_label: "prometheus"
    ha_replica_label: "prometheus_replica"
  distributor:
    ha_tracker:
      enable_ha_tracker: true
      kvstore:
        store: "consul"
        consul:
          host: consul-consul-server.apps:8500
  # ...
# ...
```

This would let Cortex know that we are running Prometheus cluster with multiple replicas. And how to handle the duplication of those samples. I set the `ha_cluster_label` to `prometheus` and `ha_replica_label` to `prometheus_replica` since these are the default values of the labels if you deployed Prometheus from the `kube-prometheus-stack` Helm chart.

With all of that, we will have a full values file below:

```yaml
#cortex_values.yaml
ingress:
  enabled: true
  ingressClass:
    enabled: true
    name: "nginx"

alertmanager:
  enabled: false
ruler:
  enabled: false

nginx:
  enabled: true
  replicas: 2

ingester:
  autoscaling:
    enabled: true
    minReplicas: 2
    maxReplicas: 5
  persistentVolume:
    enabled: true

  serviceMonitor:
    enabled: true

distributor:
  autoscaling:
    enabled: true
    minReplicas: 2
    maxReplicas: 5
  persistentVolume:
    enabled: true

  serviceMonitor:
    enabled: true

compactor:
  enabled: true
  replicas: 1

  serviceMonitor:
    enabled: true

  persistentVolume:
    enabled: true
    size: 2Gi
    storageClass: longhorn

querier:
  replicas: 2

  serviceMonitor:
    enabled: true

query_frontend:
  replicas: 2

  serviceMonitor:
    enabled: true

table_manager:
  replicas: 1

  serviceMonitor:
    enabled: true

store_gateway:
  replicas: 3

  serviceMonitor:
    enabled: true

  persistentVolume:
    enabled: true
    size: 2Gi
    storageClass: longhorn

memcached:
  enabled: false

# -- index read caching for legacy chunk storage engine
memcached-index-read:
  enabled: false
# -- index write caching for legacy chunk storage engine
memcached-index-write:
  enabled: false

memcached-frontend:
  enabled: true
  architecture: "high-availability"
  replicaCount: 2
  resources: {}

memcached-blocks-index:
  architecture: "high-availability"
  replicaCount: 2

memcached-blocks:
  architecture: "high-availability"
  replicaCount: 2

memcached-blocks-metadata:
  architecture: "high-availability"
  replicaCount: 2

config:
  storage:
    engine: blocks
    index_queries_cache_config:
      memcached:
        # -- How long keys stay in the memcache
        expiration: 1h
      memcached_client:
        # -- Maximum time to wait before giving up on memcached requests.
        timeout: 1s
  blocks_storage:
    backend: filesystem
    tsdb:
      dir: /data/tsdb
    bucket_store:
      sync_dir: /data/tsdb-sync
      bucket_index:
        enabled: true
  limits:
    ingestion_rate: 150000
    max_series_per_user: 0
    max_series_per_metric: 0
    accept_ha_samples: true
    ha_cluster_label: "prometheus"
    ha_replica_label: "prometheus_replica"
  distributor:
    ha_tracker:
      enable_ha_tracker: true
      kvstore:
        store: "consul"
        consul:
          host: consul-consul-server.apps:8500
  ingester:
    lifecycler:
      ring:
        kvstore:
          store: "consul"
          consul:
            host: consul-consul-server.apps:8500
```

Now, run the installation command:

```
helm install cortex --create-namespace --namespace cortex -f cortex_values.yaml cortex-helm/cortex
```

And boom! We have a fully scalable and distributed Prometheus!

## Cortex Grafana dashboards

Since Cortex is just a fast Prometheus-compatible monitoring system, you could use Grafana to visualize the metrics in it.

Let's set up the Helm chart again:

```
helm repo add grafana https://grafana.github.io/helm-charts
helm repo update
```

Then create another custom values file:

```yaml
# grafana_values.yaml
replicas: 1
persistence:
  enabled: false
ingress:
  enabled: false
# ...
```

You could enable the ingress and persistence.enabled if you wanted.

We could set Cortex as a Grafana data source: 

```yaml
# grafana_values.yaml
# ...
datasources:
  datasources.yaml:
    apiVersion: 1
    datasources:
    - name: Cortex
      type: prometheus
      url: http://cortex-nginx.cortex:80/prometheus
```

And that's it! Now, we run the installation command:

```
helm install grafana grafana/grafana -f grafana_values.yaml
```

From here, you could import the recommended dashboards from [monitoring.mixins.dev](https://monitoring.mixins.dev/) for Cortex at https://monitoring.mixins.dev/cortex/#dashboards

And with that, we have a Cortex monitoring system up and running. This would be the end of the part 1 of the Cortex series. See you in the next posts!
