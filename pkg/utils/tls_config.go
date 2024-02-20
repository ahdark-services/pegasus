package utils

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"os"
)

type TLSParams struct {
	CACertPath         string
	ClientCertPath     string
	ClientKeyPath      string
	InsecureSkipVerify bool
}

func NewTLSConfig(ctx context.Context, params TLSParams) (*tls.Config, error) {
	ctx, span := tracer.Start(ctx, "utils.NewTLSConfig")
	defer span.End()

	if params.CACertPath == "" && params.ClientCertPath == "" && params.ClientKeyPath == "" {
		return nil, nil
	}

	var caPool *x509.CertPool
	if params.CACertPath != "" {
		caCert, err := os.ReadFile(params.CACertPath)
		if err != nil {
			span.RecordError(err)
			return nil, err
		}

		caPool = x509.NewCertPool()
		caPool.AppendCertsFromPEM(caCert)
	} else {
		pool, err := x509.SystemCertPool()
		if err != nil {
			span.RecordError(err)
			return nil, err
		}

		caPool = pool
	}

	var clientCert tls.Certificate
	if params.ClientCertPath != "" && params.ClientKeyPath != "" {
		cert, err := tls.LoadX509KeyPair(params.ClientCertPath, params.ClientKeyPath)
		if err != nil {
			span.RecordError(err)
			return nil, err
		}

		clientCert = cert
	}

	return &tls.Config{
		RootCAs:            caPool,
		Certificates:       []tls.Certificate{clientCert},
		InsecureSkipVerify: params.InsecureSkipVerify,
	}, nil
}
