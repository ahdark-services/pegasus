package utils

import (
	"errors"
	"net"
)

var ErrNoLocalIPFound = errors.New("no local ip found")

type Addr struct {
	network string
	address string
}

func NewAddr(network, address string) net.Addr {
	return &Addr{
		network: network,
		address: address,
	}
}

func (a *Addr) Network() string {
	return a.network
}

func (a *Addr) String() string {
	return a.address
}

func GetLocalIP() (string, error) {
	interfaceAddrs, err := net.InterfaceAddrs()
	if err != nil {
		return "", err
	}

	for _, address := range interfaceAddrs {
		switch address.(type) {
		case *net.IPNet:
			ip := address.(*net.IPNet).IP
			if ip.IsLoopback() {
				continue
			}

			if ip.To4() != nil {
				return ip.String(), nil
			}
		case *net.IPAddr:
			ip := address.(*net.IPAddr).IP
			if ip.IsLoopback() {
				continue
			}

			if ip.To4() != nil {
				return ip.String(), nil
			}
		default:
			continue
		}
	}

	return "", ErrNoLocalIPFound
}
