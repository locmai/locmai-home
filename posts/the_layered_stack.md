---
title: 'The Layered Architecture'
description: "The one architecture that just simply work."
date: 2021-08-28
author:
  name: Loc Mai
tags:
  - humble
  - homelab
published: true
layout: layouts/post.njk
---

## So ... what is it???

The Humble project's code base is derived from a similar architecture design pattern that we have at my current company that we applied for both the infrastructure and the applications. The code may change from time to time but the core of it will be keeping the same.   

The main idea is to breaking down the stack into logical layers with separate responsibilities and manage dependencies. A higher layer can use services in a lower layer, but not the other way around. Therefore, the folder structure and the dependecies will look like this:

``` yaml/
┌───────────────────────────┐
│   L3. Applications        |
└─────────────▲─────────────┘                  
              │                                
┌─────────────┴─────────────┐                  
│     L2. Platform          ◄──────────────────┐
└─────────────▲─────────────┘                  │
              │                                │
┌─────────────┴─────────────┐                  │
│   L1. Infrastructure      ◄──────────────────┤
└─────────────▲─────────────┘                  │
              │                                │
┌─────────────┴─────────────┐   ┌──────────────┴────────────┐
│       L0. Metal           ◄───┤     LE. Third-parties     │ 
└───────────────────────────┘   └───────────────────────────┘
```

So from lower to higher, we have 4 layers, from L0 to L3 (cuz we're hipsters and we count from 0). There's also an extra layer called third-parties. It should have been called Global, but with the absent of Cloud infrastructures, I have no idea for the new official layer name.

``` yaml/
tree -L 2
.
├── Makefile
├── apps
│   ├── Makefile
│   └── argocd
├── docs
│   └── checklist.md
├── infras
│   ├── Makefile
│   ├── ansible
│   └── terraform
├── metal
│   └── ansible
├── platform
│   ├── Makefile
│   ├── argocd
│   └── terraform
├── scripts
└── tests
```

## Deep dive

Let's go through one by one and see what they (will) do and have.

### Layer 0: Metal

![Having a metal arm is awesome](https://imagedelivery.net/34xh1sPWPAwO1lv63pW2Eg/f7110adb-b689-4ed8-940c-2be4a046b600/public)

One must have something made of metal to be awesome, this layer provides Ansible playbooks for preparing the following:

- **OS-level packages installation**: Trying to keep this minimal, only the required packages and some troubleshooting tools. 
- **ETCD**: Centralized backend for Terraform. Running in clusterting mode accross my 3 control plane nodes.
- **CloudFlare DDNS setup**: CloudFlare is one of third-party service I used for programmatically managing DNS and hosting this blog!
- **Netdata agent setup**: For OS-level monitoring, going to replace this setup with Prometheus when I have a complete list of features required for bare metal monitoring.

This is the specs of the bare metal servers: (From my co-worker homelab, I have a pretty much same shit without the 256GB SSD per node)

``` yaml/
4 nodes of NEC SFF PC-MK26ECZDR (Japanese version of the ThinkCentre M700):
- CPU: Intel Core i5-6600T @ 2.70GHz
- RAM: 16GB

TP-Link TL-SG108 switch:
- Ports: 8
- Speed: 1000Mbps
```

### Layer 1: Infrastructure

In this layer, we have Ansible for spinning up the DDNS and Terraform for spinning up the whole cluster as well as some prerequisite resources that related to the infrastructure needed for all the platform applications and normal applications to run on. For example like both platform and appication layers will benefit from the load balancer features from MetalLB.

The Ansible stack Just contain a playbook for DDNS setup.

The Terraform stack would:

- **Spin up a Kubernetes cluster**: Using the RKE provider. 
- **Initilize resources**: Prepare for the next layers those need 
  - ArgoCD for GitOps and continuous delivery.
  - MetalLB and Nginx for network load balancing.
  - GitHub Webhook for ArgoCD auto-triggering.
  - Longhorn for persistent block storage on Kubernetes.
  - And Vault for ... I can't tell, it's a secret.

### Layer 2: Platform

The platform layer provides the applications/tools that (assume there are) developers in a team would need. Those are in the following categories:

- **CI/CD**: 
  - ArgoCD
  - GitHub Action
- **Observability**:
  - Prometheus: monitoring system/database for storing time-series data.
  - Loki: a multi-tenant log aggregation system inspired by Prometheus, this is still in beta, go pretty well with Grafana for Log viewing.
  - Jaeger: support collect and query tracing spans.
  - Grafana: Centralized BI tool for querying and drawing graph from the collected data from the above sources.
- **Quality Checking**:
  - SonarQube: Code quality checking.
  - Sitespeed.io: Front-end performance synthesis tool, checking the front-page of all the applications.

### Layer 3: Applications

This is a straight forward layer contains my self-development applications deployed via ArgoCD.

The ArgoCD provides the declarative setup style, so I specified one app that consists a Helm-based app that would deploy other apps. This approach enables dynamically templating the applications' manifests. And not just Helm-based apps, we could also use this for other tooling that ArgoCD provided.

So the Helm-based chart looks like:

``` yaml/
.
├── Chart.yaml
├── main.yaml
├── templates
│   ├── blackping.yaml
│   ├── keycloak.yaml
│   ├── ory-kratos.yaml
│   └── postgresl.yaml
└── values.yaml
```

Where everything could be manipulated by the main values.yaml, and everything could be made parameterized.

For example, I made the blackping app to have a switch that turn on and off the app:

``` yaml/
# ./templates/blackping.yaml
{{ '{{- if .Values.blackping.enabled }}'}}
...
{{ '{{- end }}'}}
```

And in the main values.yaml, to turn it on, specify `blackping.enabled` value:

``` yaml/
blackping:
  enabled: true
```

The apps are also benefits from the lower layers:

Fetching PostgreSQL's secrets from Vault:

``` yaml/
# ./templates/ory-kratos.yaml
annotations:
  vault.hashicorp.com/agent-inject: "true"
  vault.hashicorp.com/agent-inject-secret-config: 'secret/postgresql/data'
  vault.hashicorp.com/role: "postgresql_read_only"
  vault.hashicorp.com/agent-inject-template-config: |
    {{'{{`{{ with secret "secret/postgresql/data" -}}
      export DSN="postgres://{{ .Data.data.kratos_username }}:{{ .Data.data.kratos_password }}@postgresql:5432/{{ .Data.data.kratos_database }}?sslmode=disable&max_conns=20&max_idle_conns=4"
    {{- end }}`}}'}}
```

### Layer Extra: Third-parties

I'm using a couple of third-party services that I don't find a better one to replace for now.

**GitHub**: One of the best remote VCS provider. It easily integrates with GitHub Actions, GitHub Project board and other platform tools like ArgoCD.

**CloudFlare**: I planned to shoutout for CloudFlare a long time ago for their wonderful services, they provide the best global network for any websites, especially their free plan is out of this world for self-development. 

My blog is built and hosted on CloudFlare, this is their status:

<img alt="CloudFlare is a beast" src="https://imagedelivery.net/34xh1sPWPAwO1lv63pW2Eg/9b5b6e81-ac28-48fb-1ca2-40c941364100/public" width="30%" height="30%">

The blog page above (picture captured on Sept 29th) is also surviving one big outage from Fastly this year that took down several big services like GitHub, StackOverflow but not ... uh uh CloudFlare!

**Better Uptime**: Ermm ... they gave me the status page and the ping checking. I don't complain on that :)

**Bitwarden**: an open-source password manager. I learned that from my current work that I need at least one password manager. This comes with browser extension, mobile application, desktop and CLI applications.

## In the end

I hope this blog post could describe a bit about the Humble project's architecture design, why and how things were putting in there. I will go explain the details if needed. Until Then!
