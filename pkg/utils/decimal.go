package utils

import "github.com/shopspring/decimal"

func MoneyDecimal(money int64) decimal.Decimal {
	return decimal.NewFromInt(money).Div(decimal.NewFromInt(100)).Round(2)
}

func DecimalMoney(dec decimal.Decimal) int64 {
	return dec.Mul(decimal.NewFromInt(100)).Round(0).IntPart()
}

func PercentDecimal(percent int64) decimal.Decimal {
	return decimal.NewFromInt(percent).Div(decimal.NewFromInt(10000)).Round(4)
}

func CalculateFee(percent int64, fixed int64, amount decimal.Decimal) decimal.Decimal {
	return MoneyDecimal(fixed).Add(amount.Mul(decimal.NewFromInt(percent).Div(decimal.NewFromInt(10000))))
}
