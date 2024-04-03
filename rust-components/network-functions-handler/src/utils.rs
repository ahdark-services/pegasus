use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use lazy_static::lazy_static;
use opentelemetry::{Context, global, KeyValue};
use opentelemetry::trace::{TraceContextExt, Tracer};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

lazy_static! {
    static ref RESOLVER: TokioAsyncResolver =
        TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
}

pub(crate) async fn parse_target(parent_cx: Context, target: &str) -> anyhow::Result<IpAddr> {
    let tracer = global::tracer("pegasus/rust-components/network-functions-handler/utils");
    let cx = parent_cx.with_span(
        tracer
            .span_builder("parse_target")
            .start_with_context(&tracer, &parent_cx),
    );

    cx.span()
        .set_attribute(KeyValue::new("target", target.to_string()));

    if let Ok(ip_addr) = target.parse::<Ipv4Addr>() {
        Ok(IpAddr::V4(ip_addr))
    } else if let Ok(ip_addr) = target.parse::<Ipv6Addr>() {
        Ok(IpAddr::V6(ip_addr))
    } else {
        let ip_addresses = RESOLVER.lookup_ip(target).await?;

        while let Some(ip_address) = ip_addresses.iter().next() {
            if ip_address.is_loopback() {
                continue;
            }

            return match ip_address {
                IpAddr::V4(ip_addr) => Ok(IpAddr::V4(ip_addr)),
                IpAddr::V6(ip_addr) => Ok(IpAddr::V6(ip_addr)),
            };
        }

        Err(anyhow::anyhow!("Failed to resolve IP address"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_target() {
        assert_eq!(
            parse_target(Context::default(), "127.0.0.1").await.unwrap(),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        );

        assert_eq!(
            parse_target(Context::default(), "::1").await.unwrap(),
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
        );

        assert!(parse_target(Context::default(), "example.com")
            .await
            .is_ok());
    }
}
