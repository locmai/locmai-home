---
title: 'How to structure Helm code'
description: "Simpliest way to write deployable Helm code"
date: 2021-09-02
author:
  name: Loc Mai
tags:
  - helm
  - kubernetes
layout: layouts/post.njk
---

## Isn't it just ... templating?

True. I don't really like the way they defined Helm as a 'package manager' for Kubernetes rather than a templating tool - a great one - throughout my several projects working with it, I've learned a few things that I'd love to share:
- How to keep your values.yaml file minimal.
- Simplify the multi-deloyments model but still keep the dynamic of templating.
- Better ways to do things with Helm.

I will go through a couple of basic Helm patterns and examples before giving my summary on how should we do this one properly.

## Helm Patterns 

### The basic

Let's begin with the basic one:

``` yaml/
mychart/
  Chart.yaml
  values.yaml
  charts/
  templates/
    - _helpers.tpl
    - service.yaml
    - deployment.yaml
    - ...
```

First off, this is an example of single chart, deploy single deployment with service.

We have `Chart.yaml` is a file contains a description of the chart. The file also define the neccessary dependencies (other charts) to pull in the `charts` folder and use their code.

The `values.yaml` is the default input values that could be override later. I find this one is better for documentation than most of the README.md in the public repository, take a look at [Keycloak chart's values.yaml](https://github.com/bitnami/charts/blob/master/bitnami/keycloak/values.yaml) and you will see the default values, the example values that could be inputted into the chart. So when I use any chart from any vendor, I always look at their values.yaml file first.

In the templates folder, there are Kubernetes manifests that may contain the templating logic.

So if we'd start writing a new chart, this is the most basic and most minimal chart structure.

### Application/Library Type

Helm v3 introduced the new chart type called library which helps reducing the helm chart boilerplate code. As they started to realized more and more duplicated templating code have been written so far since the first days. They are different in the way we think of them: Application is for application related resources we would want to deploy, Library is for utilities/helper tools that supporting writing the manifests.

For example, we could define a common library chart with the following ./templates/_templatevalues.tpl

``` yaml/
{{'{{/* vim: set filetype=mustache: *}}
{{/*
Renders a value that contains template.
Usage:
{{ include "common.tplvalues.render" ( dict "value" .Values.path.to.the.Value "context" $) }}
*/}}
{{- define "common.tplvalues.render" -}}
    {{- if typeIs "string" .value }}
        {{- tpl .value .context }}
    {{- else }}
        {{- tpl (.value | toYaml) .context }}
    {{- end }}
{{- end -}}'}}
```

With the above code, we are now be able to port the input in the values.yaml directly into the manifest under YAML format in other charts.

### Parent Chart/Subcharts

One of the core features of Helm is Parent chart and Subchart. We could define the subcharts as dependencies of a parent chart under `dependencies` field in the Chart.yaml as follow:

``` yaml/
# Chart.yaml
dependencies:
- name: nginx
  alias: nginx-something
  version: "1.2.3"
  repository: "https://example.com/charts"
- name: memcached
  alias: memcached-index-read
  version: "3.2.1"
  repository: "https://charts.bitnami.com/bitnami"
  condition: memcached.enabled
- name: memcached
  alias: memcached-index-write
  version: "3.2.1"
  repository: "https://charts.bitnami.com/bitnami"
  condition: memcached.enabled
```

A few things we could describe for the dependencies are:
- name: the name of the chart
- alias: the alias name in case you wanna use the same chart for different purposes.
- version: Helm come with versioning feature as a package manager, this would help if your other charts need to be consist to one version before upgrading to a newer version.
- condition: An official way to turn on/off the dependency.


And with just that, you could override any value of the subchart by writing the name as the first key, and then the normal input. Let's say I want to change the memcached-index-read's `image.tag`, in the values.yaml of the parent chart, I could do:

``` yaml/
memcached-index-read:
  image:
    tag: 1.6.10-debian-10-r0
```

Pretty simple. The drawback of this method is that you could only set the static value. So it wouldn't work if the values is dynamic/generated from other sources. For that case, the only way you could do is go to the subchart and propose override code for it.

I use the same pattern for my old company - with the structure something like this:

``` yaml/
apps/
  Chart.yaml
  values.yaml
  charts/
  templates/
    - _helpers.tpl
    - shared_resources.yaml
```

The Chart.yaml's dependencies looks like this:

``` yaml/
dependencies:
- name: generic-service
  alias: service-core
  version: "x.y.z"
  repository: "https://private.repo/charts"
  condition: service-core.enabled
- name: generic-service
  alias: service-scheduler
  version: "x.y.z"
  repository: "https://private.repo/charts"
  condition: service-scheduler.enabled
- name: generic-service
  alias: service-access-control
  version: "x.y.z"
  repository: "https://private.repo/charts"
  condition: service-access-control.enabled
```

This way, I could have a generic service (Application) chart and re-use it over and over again. The values.yaml would define the different:

``` yaml/
service-core:
  enabled: true
  image:
    name: service-core 
    tag: 1.6.0
  replicas: 5

service-scheduler:
  enabled: true
  image:
    name: service-scheduler
    tag: 1.1.2
  
  replicas: 2

service-access-control:
  enabled: true
  image:
    name: service-access-control
    tag: 1.2.2
  
  replicas: 2
```

This is great, but not enough.

If you had some values that shared across the subcharts and don't want to specify them twice in the values.yaml, you could use the Global Values as

``` yaml/
global:
  region: westeurope
```

And all other charts could use it as 

``` yaml/
{{ '{{' }} Values.global.region {{ '}}' }}
```

### ArgoCD Apps with Helm

This is not pure Helm but with a helping hand from ArgoCD, it would be much easier.

For anyone who doesn't know what ArgoCD is, [it's a declarative GitOps tool](https://argoproj.github.io/argo-cd/) that help deploying Kubernetes stuffs.

ArgoCD provides a custom k8s resource called `Application` that could deploy a Helm chart by defining this manifest and let ArgoCD handles the rest:

``` yaml/
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: argocd-ory-kratos
  namespace: argocd
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: argocd
  syncPolicy:
    automated:
      selfHeal: true
      prune: true
    syncOptions:
      - CreateNamespace=true
  destination:
    name: in-cluster
    namespace: apps
  source:
    chart: kratos
    repoURL: https://locmai.github.io/ory-k8s/
    targetRevision: master
    helm:
      releaseName: ory-kratos
      values: |
        kratos:
          development: false
```

So imagine we could put this manifest in the templates folder of a chart

``` yaml/
.
├── Chart.yaml
├── templates
│   └── ory-kratos-app.yaml
└── values.yaml
```

And update it with the Helm templating code:

``` yaml/
{{'{{- if .Values.ory_kratos.enabled }}
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: {{ .Values.argocd.project }}-ory-kratos
  namespace: {{ .Values.argocd.namespace }}
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: {{ .Values.argocd.project }}
  syncPolicy:
    automated:
      selfHeal: true
      prune: true
    syncOptions:
      - CreateNamespace=true
  destination:
    name: in-cluster
    namespace: {{ .Values.argocd.project }}
  source:
    chart: kratos
    repoURL: https://locmai.github.io/ory-k8s/
    targetRevision: {{ .Values.ory_kratos.targetRevision }}
    helm:
      releaseName: ory-kratos
      values: |
        kratos:
          development: false
{{- end }}'}}
```

Now, we could do enabled: true/false just like the Parent/Subchart pattern, and also make the values inside more dynamic.

``` yaml/
ory_kratos:
  targetRevision: 0.9.10
  enabled: true
```

The one problem left is that how to explain to your boss the way we templated the template to deploy things called applications that deploy more templates.

Appception.

### Bottom-up

Defining the single-purpose base chart structure that simple enough to understand/override/extend. On that you could write various charts. Also start splitting the logical charts that could reuse for other charts and called them library charts.

For the choice between Parent/Subchart or the ArgoCD, consider how dynamic your values are. I'd go straight parent/subchart if they were plain static. Otherwise, deploy ArgoCD and also apply GitOps practice is not a bad idea at all. Keep in mind that we only want one layer of parent and children, adding 'grandparent' would lead the code to become more complex to understand and harder for debugging what went wrong during the templating process.
