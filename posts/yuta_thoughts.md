---
title: 'Yuta - Train of Thought'
description: "Train of thought about the development of Yuta"
date: 2022-03-18
author:
  name: Loc Mai
tags:
  - random
layout: layouts/post.njk
---

I recently began the Yuta project - earlier than intended - because Humble has reached the stage I desired and is suitable for my development playground. The details should be posted [here](https://maibaloc.com/projects/yuta/) later on I guess. For the time being, I just want to jot down a few notes from the process thus far.

I'm used to getting off course from my original plans, so this time I'm going to set a clear objective for what I want to accomplish and when the project will be completed. 

So the main objective is to build **an operation bot** that lives in the system to help maintain and monitor the whole system from the inside. The name and image inspired by [Yuta from Tonton friends webcomic](https://tontonfriends.fandom.com/wiki/Yuta). 

Yuta may understand users' intents from the messages sent to him and able to take any action if required. For example, if I wanted to scale up new pods in my cluster, I could ask Yuta to do it. Or I could ask for the documents of any known projects to him.

Some fancy buzzwords and keywords popped up in my head at the time were: natural language understanding, Kubernetes-native, system design, micro-omega-hyper-ultra-super-speed-services? So here is a bunch of random thoughts: 

**Micro-services?** Not really a big fan, but with my experiences with the old version of Yuta (full monolith), scaling is an issue. Also, tangled components make it extremely hard to extend.

**Polylith?** This architecture design I read about a few weeks ago when I went through matrix-org/dendrite code. Nah, just another buzzword for some hybrid monolith-micro-services stuffs. But I like the shared components/modules thing.

**Kubernetes-native** Sure, I love Kubernetes. I wrote a bunch of Kuberenetes tools with the golang/python clients.

**Natural Language Understanding** Hmm ... for the previous Yuta, I wrote a Songoku Kamehameha Machine Learning algorithm in order for it to comprehend what humans are saying. Something brilliant like this:

```python
if "scale" in human_message:
  if "helloworld" in human_message:
    if "default namespace" in human_message:
      replicas_number = found_number_to_scale(human_message)
      if found_number_to_scale(human_message):
        # bunch of other text extracting code
        if i_know_how_to_write_code_better:
          deployment_api.update(replicas=replicas_number)
          send_message("deployment updated")
else:
  pass # passing out writing this stuff at line 1069
```

So yeah, for this time, I will delegate that part for any NLU platforms like the Diaglogflow or Luis. I just need the intents and the entities came from the dialogues.

**Active and Proactive** The active part of Yuta is that I as a user could demand Yuta to take an action that I intended. To make it different than the other run-this-command bot, Yuta should be able to react to what is going on in the system in a proactive manner. How about making it monitoring the system via something like Prometheus and detect the anomaly events?

**PAD** Prometheus Anomaly Detection. Meh, I don't have much idea about this field. But just put it here to make the idea looks cool.

And with all of those keywords, how do I put up with the code base? Which ones should I drop? So I have spent eternity sitting in the toilet to think about the project.

We will have the following components:

**core** The core component is a server that run any integrated app services (kubeops, argocd, github, etc). `core` server would consume the messages in the NATS jetstream, these messages produced by the other two components.

**messaging** The messaging component will handle most of the chat messages logic with the chat platform and the NLU platforms then extract the intents and entities into actionable items, then put those items as messages into NATS jetstream so `core` could act on them.

**pad** The pad component will continuously monitor and get the metrics from Prometheus, detect any anomaly events and put similar actionable items into the message stream.

**common** All the common configuration/code that will be used in most of the components.

Let's draw a big diagram here where things glued up altogether:

```
             +----------------+         +------------------+
             |                |         |                  |
             |  chat clients  |         |    nlu clients   |
             |  matrix,slack  |         | diaglogflow,luis |
             |                |         |                  |
             +-------^--------+         +---------^--------+
                     |                            |
                     |                            |
                     |                            |
+---------------+    |                            |
|               |    |                            |
|    common     |    |                            |
|               |    +-----+--------------+-------+
+-------+-------+          |              |
        +------------------>   messaging  +-------+-------+
        |                  |              |       +-------v-----+
        |                  +--------------+       |    nats     |
        |                                         |  jetstream  |
        |                  +--------------+       |             |
        |                  |              |       +------^----^-+
        +------------------>     core     |              |    |
                           |              +--------------+    |
                           +-------+------+                   |
                                   |                          |
                                   |                          |
                                   |     +----------------+   |
                                   |     |                |   |
                                   +----->    kubeops     |   |
                                   |     |                |   |
                                   |     +----------------+   |
                                   |                          |
                                   |     +----------------+   |
                                   |     |                |   |
                                   +----->     argocd     |   |
                                   |     |                |   |
                                   |     +----------------+   |
                                   |                          |
                                   +-----> ...                |
                                                              |
+--------------+           +--------------+                   |
|              |           |              |                   |
|  prometheus  +----------->     pad      |                   |      
|              |           |              +-------------------+
+-------+------+           +-------+------+
```

With all that, I set up the code base, wrote some code and here is the first demo of Yuta:

[![yuta-first-demo](https://img.youtube.com/vi/kOjw9wWp9aU/0.jpg)](https://www.youtube.com/watch?v=kOjw9wWp9aU)
