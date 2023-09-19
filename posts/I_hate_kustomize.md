---
title: 'I hate kustomize'
description: "A rant post about the marvelous kustomize"
date: 2023-03-23
author:
  name: Loc Mai
tags:
  - kustomize
layout: layouts/post.njk
---

## The title is a clickbait

No it's not. I took it personally not because how "bad" the tool is but how they
claimed it to be better than Helm but unable to completely resolve the real
issues. And I don't even like Helm that much or saying Helm is the best or
anything else to be honest.

## We touch it, but not really, but still touch it.

Take a look at the first claim at the landing page https://kubectl.docs.kubernetes.io/:

> Kustomize lets you customize raw, template-free YAML files for multiple
> purposes, leaving the original YAML untouched and usable as is.

Cool. Awesome. Lean, mean, Kubernetes-manifest generator machine. Wait how do
you do the customization without touching the raw YAML??? Ohhhh by having the
patching functions to touch it in more weird ways: patch, patchesStrategicMerge,
~patchJson6969~ patchJson6902, etc.

Do you even read your examples? How many JSON or YAML multi-line string
should I write to set a service rule right??

```
[
  { "op": "replace", "path": "/spec/rules/0/host", "value": "foo.bar.io" },

  {
    "op": "replace",
    "path": "/spec/rules/0/http/paths/0/backend/servicePort",
    "value": 80
  },

  {
    "op": "add",
    "path": "/spec/rules/0/http/paths/1",
    "value": { "path": "/healthz", "backend": { "servicePort": 7700 } }
  }
]
```

Ref:
[https://github.com/kubernetes-sigs/kustomize/blob/master/examples/jsonpatch.md](https://github.com/kubernetes-sigs/kustomize/blob/master/examples/jsonpatch.md)

Well I guess it's not that bad to patch multiple objects with just YAML right?
Oh, more reading about the `patches`:

```
patches:
  - path: <relative path to file containing patch>
    target:
      group: <optional group>
      version: <optional version>
      kind: <optional kind>
      name: <optional name or regex pattern>
      namespace: <optional namespace>
      labelSelector: <optional label selector>
      annotationSelector: <optional annotation selector>
```

Ref:
[https://github.com/kubernetes-sigs/kustomize/blob/master/examples/patchMultipleObjects.md](https://github.com/kubernetes-sigs/kustomize/blob/master/examples/patchMultipleObjects.md)

Brilliant! Now instead of just considering where the field is and what
value should I put, I woul have to consider what relative path to the file that I should update? And all of the internal specification? Cool feature!

## No, templating sucks. We have transformers, generators, patch strategies.

Ref: https://kubectl.docs.kubernetes.io/references/kustomize/glossary/#kustomize

Yeah. We don't need the frustation come from an engine that helps updating the current manifests.

We have the tools for transforming and generating them.

I could clearly see the differences here and the advantages of not using the word "template" but "transform" and "generate" instead.

Different. But same-same. But still different..

## A sub-whatever is not a thing. Base/overlay structure is kinda similar but much better.

```
sub-target / sub-application / sub-package
A sub-whatever is not a thing. There are only bases and overlays.
```

Ref: https://kubectl.docs.kubernetes.io/references/kustomize/glossary/#sub-target--sub-application--sub-package

First of all, the base/overlay structure is not even comparable to the sub-thing here. The base and overlay relationship of Kustomize is equal to the templates/values relationship of Helm.

```
An overlay is a kustomization that depends on another kustomization.
```

Yes, a Helm value file is just a simple YAML file that depends on other Helm files called "templates"

```
An overlay is unusable without its bases.
```

Sounds similar, right? You can't do shit with values.yaml unless you have the templates.

```
Overlays make the most sense when there is more than one, because they create different variants of a common base - e.g. development, QA, staging and production environment variants.
```


I don't see any reason why can't the values.yaml file could have more than one? Heard of values-dev.yaml, values-prod.yaml or even have them in a folder and call that folder an overlay component is okay right?

And it's all about the bass ... I mean the base:

```
A base is a kustomization referred to by some other kustomization.
Any kustomization, including an overlay, can be a base to another 
kustomization.
```

Yes, again, NOT a-sub-module or sub-overlay BUT can be a base of another `kustomization`. I see what you did there.

## The word package has no meaning in kustomize

```
The word package has no meaning in kustomize, as kustomize is not to be confused with a package management tool in the tradition of, say, apt or rpm.
```

Sure, this is why I see this implementation a lot:

```
|_ kustomization.yaml
|_ helmrelease.yaml
```

Where `kustomization.yaml` is like:

```
kind: Kustomization
namespace: monitoring
resources:
  - ./helmrelease.yaml
```

And Grafana eventually deployed by Flux through Helm release:

```
apiVersion: helm.toolkit.fluxcd.io/v2beta1
kind: HelmRelease
metadata:
  name: grafana
  namespace: monitoring
spec:
  interval: 30m
  chart:
    spec:
      chart: grafana
      sourceRef:
        kind: HelmRepository
        name: grafana
  values: ...
```

I see how useful kustomize is in this scenario where you could have the awesome kustomization.yaml file there! Woohoo, what an innovation!

## Import globbing is bad, and why tooling that expands imports is superior

Ref: https://kubectl.docs.kubernetes.io/faq/kustomize/eschewedfeatures/#globs-in-kustomization-files

2 contributors from the project claimed that it is bad to have globbing in the code base, they referrenced a Java blog post and then removed the feature from kustomize 5 years. The PR has no description on why they did that, 59 thumbs down, and more followed-up issues.

2 years later after the PR merged, the same contributor with the Java blog post said:

```
It's become clear over time that kustomize is primarily used in a git context.

When that's the case, questions about what went into a build are moot, and a resources: field can be globbed or even eliminated as suggested in #3204 (another issue).
```

So um ... I guess 5 years ago we didn't have Git, Terraform, or any good example for him/her/them to think that we would run this in Git context? But there is Java and because of the kustomize should remove the feature?  

3 years later after the PR merged, the contributors agreed on the needs for the feature but due to "maintainer overload" issue, they could no longer do anything.

But I think you are right! We should remove globbing from bash and other scripting/programming languages, too! 

Fricc the comments on the issues about how the users really want it, fricc the users as well, let's add more generator & transformers like Autobot and Decepticon!
