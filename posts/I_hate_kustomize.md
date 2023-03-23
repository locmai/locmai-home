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

Take a look at the first claim:

> Kustomize lets you customize raw, template-free YAML files for multiple
> purposes, leaving the original YAML untouched and usable as is.

Cool. Awesome. Lean, mean, Kubernetes-manifest generator machine. Wait how do
you do the customization without touching the raw YAML??? Ohhhh by having the
patching functions to touch it in more weird ways: patch, patchesStrategicMerge,
~patchJson6969~ patchJson6902, etc.

Dude, do you even read your examples? How many JSON or YAML multi-line string
should I write to set a service rule right??

```json
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

Ref #1:
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

Ref #2:
[https://github.com/kubernetes-sigs/kustomize/blob/master/examples/patchMultipleObjects.md](https://github.com/kubernetes-sigs/kustomize/blob/master/examples/patchMultipleObjects.md)

Brilliant! Now instead of just considering where the field is and what
value should I put, I woul have to consider what relative path to the file that I should update? And all of the internal specification? Cool feature!

## No, templating sucks. We have transformers, generators, patch strategies.

Yeah. We don't need the frustation come from an engine that helps updating the current manifests.

We have the tools for transforming and generating them.

I could clearly see the differences here and the advantages of not using the word "template" but "transform" and "generate" instead.

Different. But same-same. But still different..

## A sub-whatever is not a thing. Base/overlay structure is kinda similar but much better.

> sub-target / sub-application / sub-package
> A sub-whatever is not a thing. There are only bases and overlays.

Ref: https://kubectl.docs.kubernetes.io/references/kustomize/glossary/#sub-target--sub-application--sub-package

First of all, the base/overlay structure is not even comparable to the sub-thing here. The base and overlay relationship of Kustomize is equal to the templates/values relationship of Helm.

> An overlay is a kustomization that depends on another kustomization.

Yes, a Helm value file is just a simple YAML file that depends on other Helm files called "templates"

> An overlay is unusable without its bases.

Sounds similar, right? You can't do shit with values.yaml unless you have the templates.

> Overlays make the most sense when there is more than one, because they create different variants of a common base - e.g. development, QA, staging and production environment variants.


I don't see any reason why can't the values.yaml file could have more than one? Heard of values-dev.yaml, values-prod.yaml or even have them in a folder and call that folder an overlay component is okay right?

And about the bass ... I mean the base:

> A base is a kustomization referred to by some other kustomization.
> Any kustomization, including an overlay, can be a base to another 
kustomization.

Yes, not a-sub-base or sub-overlay. But can be a base of another `kustomization`. I see what you did there.

...
