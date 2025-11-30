# v2-uri-parser

V2ray URI parser for xray core

Currently supports: `vless`, `vmess`, `shadowsocks`, `trojan` and `socks`

```
Parses V2ray URI and generates JSON config for xray

Usage: v2parser [OPTIONS] <uri>

Arguments:
  <uri>  V2ray URI to parse

Options:
      --socksport <PORT>  Optional SOCKS5 proxy port for inbound
      --httpport <PORT>   Optional HTTP proxy port for inbound
      --get-metadata      Only print config meta data
      --run                 Run xray-core with the generated config
      --xray-binary <PATH>  Path to xray-core binary (default: xray from PATH)
  -h, --help              Print help
  -V, --version           Print version
```


### Install as Windows Task to start automatically
Run this command via PowerShell
```
powershell -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/house-of-vanity/v2-uri-parser/main/scripts/win-service-installer.ps1 -OutFile $env:TEMP\v2.ps1; & $env:TEMP\v2.ps1"
```
