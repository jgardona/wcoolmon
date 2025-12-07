# Water Cooler Monitor
<div align=center>
<img src="img/watercooler.png" width=50%/>
</div>

**Water Cooler Monitor** is a simple linux service to send temperature information in C⁰ to your water cooler display. Its build to fix the absence of software support for linux systems.

* **How to use:** Using wcoolmon -h command line will print the help below with many command options.
```
❯ wcoolmon -h
Usage: wcoolmon [OPTIONS]

Options:
  -v, --vendor-id <VENDOR_ID>    HID device vendor Id [default: 43656]
  -p, --product-id <PRODUCT_ID>  HID device product Id [default: 34406]
  -i, --interval <INTERVAL>      Time interval to update temperature info [default: 1000]
  -r, --verbose                  Verbose. Prints temperature info in C⁰ to stdout
  -h, --help                     Print help
  -V, --version                  Print version

```

* You need to list your **HID Devices using lsusb** and indentify your cooler pid(product_id) and vid(vendor_id).
```bash
❯ lsusb
Bus 001 Device 001: ID 1d6b:0002 Linux Foundation 2.0 root hub
Bus 001 Device 002: ID aa88:8666 东莞铭研电子科技 温度显示HID设备 **
Bus 002 Device 001: ID 1d6b:0003 Linux Foundation 3.0 root hub
Bus 003 Device 001: ID 1d6b:0002 Linux Foundation 2.0 root hub
Bus 003 Device 002: ID 0403:6001 Future Technology Devices International, Ltd FT232 Serial (UART) IC
Bus 004 Device 001: ID 1d6b:0003 Linux Foundation 3.0 root hub
Bus 004 Device 002: ID 0bda:b82c Realtek Semiconductor Corp. 802.11ac NIC
```

After run wcoolmon with the necessary options options above and fill your vendor id and product id water cooler hardware:
```bash
❯ sudo wcoolmon -v<vendor_id> -p<product_id>
```

* **Installation:** First of all, you need to clone this repository and compile *wcoolmon*. Its a *Rust* application static contained, thats it, you dont need to care with dependencies at runtime (only in compile time).
  
So, using *gh* github commandline, or git, just clone the repository:
```bash
❯ gh repo clone jgardona/wcoolmon
```

Build the **release profile** version of this service. Its configured to be high optimized at level z, and remove unecessary strings from the executable to save size. Once in wcoolmon root folder:

```bash
❯ cargo build --release
```

1- **Installing the service:**

- Copy the wcoolmon executable to ``/usr/local/bin``.
- Create a **systemd** in ``/etc/systemd/system/wcoolmon.service``
```bash
❯ nano /etc/systemd/system/wcoolmon.service
```
The service template:
```shell
   1   │ [Unit]
   2   │ Description=Water Cooler Monitor
   3   │ After=network.target
   4   │ 
   5   │ [Service]
   6   │ Type=simple
   7   │ ExecStart=/usr/local/bin/wcoolmon
   8   │ Restart=always
   9   │ User=root
  10   │ 
  11   │ [Install]
  12   │ WantedBy=multi-user.target
```
- Update the systemctl service list, start the service and check the status.

```bash
❯ sudo systemctl daemon-reload 
❯ sudo systemctl start wcoolmon.service 
❯ systemctl status wcoolmon.service 
```
You must see something like this:
```bash
❯ systemctl status wcoolmon.service 
● wcoolmon.service - Water Cooler Monitor
     Loaded: loaded (/etc/systemd/system/wcoolmon.service; enabled; preset: disabled)
    Drop-In: /usr/lib/systemd/system/service.d
             └─10-timeout-abort.conf
     Active: active (running) since Sun 2025-12-07 16:38:28 -03; 34min ago
 Invocation: 551794869fa34392b87a4521a45932bd
   Main PID: 1188 (wcoolmon)
      Tasks: 1 (limit: 18817)
     Memory: 1.8M (peak: 3.8M)
        CPU: 10.609s
     CGroup: /system.slice/wcoolmon.service
             └─1188 /usr/local/bin/wcoolmon

Dec 07 16:38:28 fedora systemd[1]: Started wcoolmon.service - Water Cooler Monitor.
Dec 07 16:38:28 fedora wcoolmon[1188]: Connected to Device: 8666
```
