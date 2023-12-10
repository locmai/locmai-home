---
title: 'Steam Deck as infrastructure control plane'
description: "Steam Deck setup as the intial control plane to spin up humble project"
date: 2023-12-10
author:
  name: Loc Mai
tags:
  - humble
  - homelab
published: true
layout: layouts/post.njk
---

## Steam Deck story

I recently returned from a pleasant trip to Canada. My cousin gave me his Steam Deck since he was seeking for the newest model, and I was looking for a Linux workstation to run as initial control plane for my humble/homelab project.

## Enable SSH

From the Steam Deck menu, switch from to the Steam Desktop. Then open up the Konsole terminal.

The terminal should look like this:

```
(deck@steamdeck ~) $
```

`deck` is the default username. By default, there is no password set for the account so you could simply run:

```
passwd
```

But my cousin set it before and luckily he still remembers the password.

Then we can enable the SSHD with sudo:

```
sudo systemctl start sshd
```

To make the SSHD after every restart, enable it:

```
sudo systemctl enable sshd

```

## Install Nix

Using `nix-shell` to install and run all the tools/packages I need for my homelab.

I followed the pretty good guide [here](https://determinate.systems/posts/nix-on-the-steam-deck).

Technically, if you don't mind breaking things, then their script is quite solid:

```
curl -L https://install.determinate.systems/nix | sh -s -- install steam-deck
```

I took the 4-steps dance:

Create /etc/systemd/system/nix-directory.service

```
[Unit]
Description=Create a `/nix` directory to be used for bind mounting
PropagatesStopTo=nix-daemon.service
PropagatesStopTo=nix.mount
DefaultDependencies=no

[Service]
Type=oneshot
ExecStart=steamos-readonly disable
ExecStart=mkdir -vp /nix
ExecStart=chmod -v 0755 /nix
ExecStart=chown -v root /nix
ExecStart=chgrp -v root /nix
ExecStart=steamos-readonly enable
ExecStop=steamos-readonly disable
ExecStop=rmdir /nix
ExecStop=steamos-readonly enable
RemainAfterExit=true
```

Then create /etc/systemd/system/nix.mount

```
[Unit]
Description=Mount `/home/nix` on `/nix`
PropagatesStopTo=nix-daemon.service
PropagatesStopTo=nix-directory.service
After=nix-directory.service
Requires=nix-directory.service
ConditionPathIsDirectory=/nix
DefaultDependencies=no
RequiredBy=nix-daemon.service
RequiredBy=nix-daemon.socket

[Mount]
What=/home/nix
Where=/nix
Type=none
DirectoryMode=0755
Options=bind
```

and create /etc/systemd/system/ensure-symlinked-units-resolve.service

```
[Unit]
Description=Ensure Nix related units which are symlinked resolve
After=nix.mount
Requires=nix-directory.service
Requires=nix.mount
DefaultDependencies=no

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/bin/systemctl daemon-reload
ExecStart=/usr/bin/systemctl restart --no-block nix-daemon.socket

[Install]
WantedBy=sysinit.target
```

Then enable the `ensure-symlinked-units-resolve` service and it will start the chain of settings:

```
sudo systemctl enable --now ensure-symlinked-units-resolve.service
```

And run the installation script normally:

```
sh <(curl -L https://nixos.org/nix/install) --daemon
```

## Install Arch Linux packages

I need docker to spin up the PXE servers. The setup is straight-forward following this guide [here](https://steamdecktips.com/blog/install-archlinux-packages-on-the-steam-deck).

Since we disabled the `steamos-readonly` after the Nix installation, I skipped this step:

```
sudo steamos-readonly disable
```

Initialize pacman's keyring:

```
sudo pacman-key --init
```

Populate pacman's keyring with the default Arch Linux keys:

```
sudo pacman-key --populate archlinux
```

And then I simply install docker with pacman. Also noticed that some packages will be uninstalled/reverted when doing the upgrade via the Steam Deck setting menu, I guess it's normal as we were messing up the internal packages.

## Some tweakings

While starting PXE boot over IPv4, I got the process hanged and the PXE servers weren't reachable for the servers.

They were blocked by the firewalld. So added a few more ports for HTTP and DHCP servers:

```
sudo firewall-cmd --add-port=80/tcp
sudo firewall-cmd --add-port=67/udp
```

## Result

I'm able to spin up the full homelab with a single command from the Steam Deck. Pretty cool and also pretty weird way of using the Steam Deck.
