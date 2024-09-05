//
//  IPOverrideWrapper.swift
//  MullvadREST
//
//  Created by Jon Petersson on 2024-02-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

public class IPOverrideWrapper: RelayCacheProtocol {
    private let relayCache: RelayCacheProtocol
    private let ipOverrideRepository: any IPOverrideRepositoryProtocol

    public init(relayCache: RelayCacheProtocol, ipOverrideRepository: any IPOverrideRepositoryProtocol) {
        self.relayCache = relayCache
        self.ipOverrideRepository = ipOverrideRepository
    }

    public func read() throws -> CachedRelays {
        let cache = try relayCache.read()
        let relayResponse = apply(overrides: ipOverrideRepository.fetchAll(), to: cache.relays)

        return CachedRelays(relays: relayResponse, updatedAt: cache.updatedAt)
    }

    public func readPrebundledRelays() throws -> CachedRelays {
        let prebundledRelays = try relayCache.readPrebundledRelays()
        let relayResponse = apply(overrides: ipOverrideRepository.fetchAll(), to: prebundledRelays.relays)

        return CachedRelays(relays: relayResponse, updatedAt: prebundledRelays.updatedAt)
    }

    public func write(record: CachedRelays) throws {
        try relayCache.write(record: record)
    }

    private func apply(
        overrides: [IPOverride],
        to relayResponse: REST.ServerRelaysResponse
    ) -> REST.ServerRelaysResponse {
        let wireguard = relayResponse.wireguard
        let bridge = relayResponse.bridge

        let overridenWireguardRelays = wireguard.relays.map { relay in
            return apply(overrides: overrides, to: relay)
        }
        let overridenBridgeRelays = bridge.relays.map { relay in
            return apply(overrides: overrides, to: relay)
        }

        return REST.ServerRelaysResponse(
            locations: relayResponse.locations,
            wireguard: REST.ServerWireguardTunnels(
                ipv4Gateway: wireguard.ipv4Gateway,
                ipv6Gateway: wireguard.ipv6Gateway,
                portRanges: wireguard.portRanges,
                relays: overridenWireguardRelays
            ),
            bridge: REST.ServerBridges(
                shadowsocks: bridge.shadowsocks,
                relays: overridenBridgeRelays
            )
        )
    }

    private func apply<T: AnyRelay>(overrides: [IPOverride], to relay: T) -> T {
        return overrides
            .first { $0.hostname == relay.hostname }
            .flatMap {
                relay.override(
                    ipv4AddrIn: $0.ipv4Address,
                    ipv6AddrIn: $0.ipv6Address
                )
            }
            ?? relay
    }
}
