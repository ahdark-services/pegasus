package areas

import (
	"context"
	_ "embed"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.opentelemetry.io/otel"
	"go.uber.org/zap"
	"math/rand/v2"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/components/areas/internal/services/areas")

type RandomAreaPO struct {
	Country string
	City    string
}

type Service interface {
	RandomArea(ctx context.Context) RandomAreaPO
}

type service struct {
	areasData []RandomAreaPO
}

func NewService() (Service, error) {
	data, err := UnmarshalAreasData(areasData)
	if err != nil {
		otelzap.L().Panic("failed to unmarshal areas data", zap.Error(err))
		return nil, err
	}

	a := make([]RandomAreaPO, 0)
	for _, v := range data {
		for _, city := range v.Cities {
			a = append(a, RandomAreaPO{
				Country: v.Country,
				City:    city,
			})
		}
	}

	return &service{a}, nil
}

func (s *service) RandomArea(ctx context.Context) RandomAreaPO {
	ctx, span := tracer.Start(ctx, "areas.service.RandomArea")
	defer span.End()

	return s.areasData[rand.UintN(uint(len(s.areasData)))]
}
