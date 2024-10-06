# recbox

suckless audio recording of your modular via an USB interface (e.g. a scarlett) with a raspberry pi, 2 systemd units and a couple lines Rust
- runs on a raspi laying around anyways
- starts recording audio after boot, just plug it into the same power strip
- use [syncthing](https://github.com/syncthing/) or scp to move recorded audio files
- or just point the out dir to any external data drive (USB stick, NAS share, etc.)
- **dependencies**: jack, cargo, syncthing (optional)

# hardware setup example

```
+------------------+        +-----------------+          +-----------------------+
|    Headphones    |        |     Monitors    |          | Raspberry Pi          |
+------------------+        +-----------------+          |                       |
          ^                          ^                   |                       |
          |                          |                   |                       |
         cue                      monitor                |                       |
          |                          |                   |                       |
+--------------------+      +---------------------+      | +-------------------+ |
|  Synth / Mixer     |----->| USB Audio Interface |----->| |  jackd            | |
+--------------------+      +---------------------+      | +-------------------+ |
                                                         |         |             |
                                                         |         v             |
                                                         | +-------------------+ |
                                                         | |  recbox           | |
                                                         | +-------------------+ |
                                                         |         |             |
                                                         |         v             |
                                                         | +-------------------+ |
                                                         | |  Audio Files      | |
                                                         | +-------------------+ |
                                                         |         |             |
                                                         |         v             |
                    +-------------------+                | +-------------------+ |
                    |   PC / Laptop     |<----- wifi-----+-|  Syncthing        | |
                    +-------------------+                | +-------------------+ |
                                                         +-----------------------+
```

# installation

```bash
# install dependencies
sudo apt update && sudo apt install -y jackd2 libjack-dev # optionally: syncthing
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# install this
cargo install recbox

# find your sound card by name
aplay -l
cat /proc/asound/cards

# configure the service files (change user, out dir and sound card)

# install and enable systemd units
sudo cp recbox/*.service /etc/systemd/system
sudo systemctl daemon-reload
sudo systemctl enable --now jackd.service
sudo systemctl enable --now recbox.service
```