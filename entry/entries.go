package entry

import (
	"go.uber.org/fx"

	"github.com/ahdark-services/pegasus/common/config"
	"github.com/ahdark-services/pegasus/common/dcron"
	"github.com/ahdark-services/pegasus/common/infra"
	"github.com/ahdark-services/pegasus/common/logging"
	"github.com/ahdark-services/pegasus/common/observability"
	"github.com/ahdark-services/pegasus/common/server"
	"github.com/ahdark-services/pegasus/common/settings"
	"github.com/ahdark-services/pegasus/common/telegram_bot"
)

func AppEntries() fx.Option {
	return fx.Options(
		config.Module(),
		logging.Module(),
		fx.WithLogger(logging.FxLogger),
		observability.Module(),
		settings.Module(),
		infra.Module(),
		dcron.Module(),
		server.Module(),
		telegram_bot.Module(),
	)
}
