# telco-vpn/adapter

Rust crate wrapping WireGuard and strongSwan IPsec management.

## Layout

- `src/lib.rs` - public API (VpnProfile, TunnelStatus)
- `src/wireguard.rs` - wg userspace integration
- `src/strongswan.rs` - strongSwan swanctl shell interface
- `src/health.rs` - tunnel status polling

## Key types

VpnProfile: encapsulates PSK, x.509 cert, peer config per site.

TunnelStatus: name, state (up | down | rekeying), packets_sent/received, bytes_sent/received.

## Build

```bash
cargo build -p eel-adapter-telco-vpn-rs --release
```

## Environment variables

- VPN_IPSEC_CONFIG_DIR: strongSwan config directory (default: /etc/ipsec.d)
- VPN_WIREGUARD_IFACE: WireGuard interface name (default: wg0)
- VPN_HEALTH_PROBE_INTERVAL_SECS: default 30

## Testing

```bash
cargo test -p eel-adapter-telco-vpn
```

Note: integration tests require root (iptables, netlink) or network namespace simulation.

## Satellite fallback

Monitors primary tunnels; if both down 5+ min, triggers satellite integration (separate module).
