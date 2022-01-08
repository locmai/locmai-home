---
title: 'Not so simple UDP proxy'
description: "My journey on how to make a UDP proxy more complicated"
date: 2021-09-09
author:
  name: Loc Mai
tags:
  - golang
  - performance
  - monitoring
published: true
layout: layouts/post.njk
---

### Context

One of my project was migrating the whole monitoring system from [DataDog](datadoghq.com) to [Cortex (a distribution of Prometheus)](https://cortexmetrics.io/). Most of our services are using DogStatsD/StatsD client to send the metrics to DataDog agent on the host, so the process required a dual-write mechanism which we have chose the plain simple approach of this [udp-proxy code of Akagi201](https://github.com/Akagi201/udpproxy). During the go-to production period, we encountered a several issues and had to extend the code.

So it was something like this:

```
                                  ┌───────────────┐        ┌─────────────┐
                                ┌─► DataDog agent ├────────► DataDog app │
                                │ └───────────────┘        └─────────────┘
┌─────────────┐   ┌─────────────┤
│ App service ├───►  udp-proxy  │
└─────────────┘   └─────────────┤ ┌───────────────┐        ┌────────┐
                                └─► OTel collector├────────► Cortex │
                                  └───────────────┘        └────────┘
```

* OTel collector - https://github.com/open-telemetry/opentelemetry-collector acts as a StatsD reciever and emits the metrics to Cortex.

### Windows build

We have some Windows hosts that running our Windows-native service. Our issue with the original udp-proxy was that Windows OS controls services by setting up callbacks very different from the other OS systems. So we need to make the service to provide the API that could answer Windows probably despite the substantial differences. So I found this service wrapper: https://github.com/kardianos/service which supports to detect how a program is called, from an interactive terminal or from a service manager, also providing the necessary API mentioned.

So I have this wrapped around the original service:

``` go/
package main

import (
	log "github.com/sirupsen/logrus"
    
	"github.com/kardianos/service"
)

var logger service.Logger

type program struct{}

func (p *program) Start(s service.Service) error {
	go p.run()
	return nil
}

func (p *program) run() {
	// udp-proxy code here
}

func (p *program) Stop(s service.Service) error {
	return nil
}

func main() {
	svcConfig := &service.Config{
		Name:        "UDPService",
		DisplayName: "UDP Service",
		Description: "UDP Proxy for dual-write metric",
	}

	prg := &program{}
	s, err := service.New(prg, svcConfig)
	if err != nil {
		log.Fatal(err)
	}

	err = s.Run()
	if err != nil {
		logger.Error(err)
	}
}
```

This one fits perfect for single build pipeline into binaries for Linux and Windows

``` bash/
go mod download
GOOS=windows GOARCH=386 go build -o build/udp-proxy-windows-$(TAG).exe .
GOOS=linux GOARCH=amd64 go build -o build/udp-proxy-amd64-linux-$(TAG) .
```

Alternative 0: we might have go with Windows container approach, but some of our Windows servers is really old, that's why we had to go with a more native Windows service approach.

Alternative 1: Non-sucking service manager - https://nssm.cc/ - is a great one. If the first one didn't work, we might have ported the [Puppet module for nssm](https://forge.puppet.com/modules/ktreese/nssm) to configure the original build as a Windows service managed by nssm and call it a day.

### Critical performance issue

For QA and Staging envs, we have a small load test for the udp-proxy and it has passed, therefore we were so confident to roll this out on production. After the roll out, we started seeing metrics being dropped, the throughput has been dropped from 14-15 million of records down to 1-2 million per metric entry.

We gathered a few things on 1 machine:

- The CPU usage from udp-proxy is peaking over 200%
- `netstat -su` will give the receive buffer errors metric, which is high AF.
- `netstat -unlp|grep -e PID -e udp-proxy` and we got the Recv-Q peaked at around 210k(MB) which matched the system network core maximum allowed for one process, check with the command `sysctl net.core.rmem_max` 

Next step, I started implementing a new load test so that it could cause the state above with JMeter, the load test is kinda simple, just looping with no sleeping time between each loop with the help of this plugin: https://jmeter-plugins.org/wiki/UDPRequest/

Boom, with a right amount of load, we could prove reproduce the case and benchmark our resource usage, then detect the bottleneck was from the following lines:

``` go/
for {
    b := make([]byte, opts.Buffer)
    n, addr, err := sourceConn.ReadFromUDP(b)

    if err != nil {
        log.WithError(err).Error("Could not receive a packet")
        continue
    }

    log.WithField("addr", addr.String()).WithField("bytes", n).WithField("content", string(b)).Info("Packet received")
    for _, v := range targetConn {
        if _, err := v.Write(b[0:n]); err != nil {
            log.WithError(err).Warn("Could not forward packet.")
        }
    }
}
```

It's a simple for loop, read the packet then simply write it to target connections. the line `v.Write` here will block the next loop since this is running in single thread.

So ...

<img alt="go solution" src="https://imagedelivery.net/34xh1sPWPAwO1lv63pW2Eg/51f538ac-49c3-49af-5916-bf812d75b500/public" width="30%" height="30%">

Leverage the best use of Go, we started refactoring and improving the code a bit, adding MetricWriter so it could handle each target connections separately:

``` go/
type MetricPacket struct {
	buffer []byte
	n      int
}

type MetricWriter struct {
	num            int
	targetAddr     *net.UDPAddr
	packetsChannel chan MetricPacket
}

func (v *MetricWriter) start() {
  // Spawn more goroutines in a controlled way to paralelize work and increase the read throughput
}
```

So now the `for` loop for reading packet is simplified to `[read packet -> put into packet channels -> next loop]`:

``` go/
for {
    b := make([]byte, opts.Buffer)

    n, _, err := sourceConn.ReadFromUDP(b)

    if err != nil {
        log.WithError(err).Error("Could not receive a packet")
        continue
    }

    for _, v := range packetsChannels {
        packet := MetricPacket{buffer: b, n: n}
        v <- packet
    }
}
```

Add a flag to limit the size of the queuing channels in the opts struct:

``` go/
var opts struct {
	Source            string   `long:"source" default:":2203" description:"Source port to listen on"`
	Target            []string `long:"target" description:"Target address to forward to"`
	LogLevel          string   `long:"log-level" description:"Log Level to use. debug,info, warn, error, fatal"`
	Buffer            int      `long:"buffer" default:"5120" description:"max buffer size for the socket io"`
	ChannelSize       int      `long:"channel-size" default:"100" description:"Set the total size of channels each writers have"`
}
```

For the Writers, we update the code to leverage the go routines features and with that, we solved the main read-blocking issue from the write. The new build passed the new load test and also solve the bottleneck on production when we rolled it out.

### Some other improvements

Lesson learned and we need to prepare for these kind of situtations in the near future, so I added a few improvements (yes, more complicated):

- Our udp-proxy is now producing it's own metrics like the packet size and counts. So we could compare the actual packets we received at the end versus the packet that udp-proxy received on the way. 
- Also have a flag to enable the above. Or disable collecting metrics from udp-proxy.
- Remember that we have peaked the Recv-Q buffer? After the write improvement, we are no longer having that issue, but to overcome that, we add a flag to configure the actual ReadBuffer from the source connection: 
  - So I added the following option to opts struct
    ``` go/
    ReadBuffer        int      `long:"readbuffer" default:"213760" description:"set connection read buffer size"`
    ```
    And the connection could be set by:
    
    ``` go/
    sourceConn.SetReadBuffer(opts.ReadBuffer)
    ```

    Then we could extend and use more than just 213760MB.
- System-level metric: similar to the self-metrics, we collect the `receive buffer errors metric` on the host and correlate it with the total packet size received. Then we may alert based on how fast the buffer errors increasing.
- Health check mechanism: I also added a really simple health checking mechanism for the purpose of deploying on Kubernetes, for now it's checking the source connection and target connections health and return back the status.

### Why didn't you 

- **Use Opentelemetry as a dual-write service since it supports DataDog as well?** At the time we rolled this out, DataDog exporter - https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/exporter/datadogexporter-  was still in-development/beta mode. It wasn't stable enough at the time and also DataDog is more investing into https://vector.dev/ ~ a similar monitoring pipeline tool like Opentelemetry.

- **Use Vector.dev instead?**  We have committed to the design before with the PoC. With a deadline ahead, we have to make it work to replace DataDog in our system. A few weeks later after committed to Opentelemetry, we found that Vector.dev is pretty awesome, I will have another post for comparison between these two.
- **Do something else better in /my humble opinions/?** Oh yes, I'd love to hear many constructive feedbacks and other solutions, mine surely won't be the best but it's a great learning process to me after all.
