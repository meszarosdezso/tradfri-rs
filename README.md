# TRÅDFRI

`coap-client` wrapper for controlling IKEA TRÅDFRI devices.

## Usage

1. Install `coap-client` using [the install script](./install_coap.sh).
2. Find the IP address and the security code (on the back) of your gateway.
3. Authenticate your client

```rs
use std::net::Ipv4Addr;
use tradfri::gateway::Gateway;

const IP: Ipv4Addr = Ipv4Addr::new(192, 168, 0, 7);
const SECURITY_CODE: &'static str = "<SECURITY_CODE>";

...
let mut gateway = Gateway::new(IP, SECURITY_CODE);
if let Err(err) = gateway.authenticate("<username>") {
    println!("Already authenticated.")
}
```

4. Switch the lights!

```rs
use std::thread;
use std::time::Duration;

...
let devices = gateway.get_device_ids().unwrap();
println!("{} devices found: {:?}", devices.len(), devices);

let device = gateway.get_device_by_id(devices[1]).unwrap();

device.turn_off().unwrap();
thread::sleep(Duration::from_secs(2));
device.turn_on().unwrap();
```

## Disclaimer

Incredibly hacky and a work in progress (:

_2022 © MIT License_
